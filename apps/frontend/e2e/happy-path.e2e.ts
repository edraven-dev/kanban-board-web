import { expect, type Locator, type Page, test } from "@playwright/test";

/**
 * Full-stack happy path against a running frontend + backend + isolated
 * Postgres (see playwright.config.ts). Persistence is re-asserted after a
 * reload at each meaningful step.
 */

function region(page: Page, name: string): Locator {
  return page.getByRole("region", { name, exact: true });
}

async function columnOrder(page: Page): Promise<string> {
  const names = await page
    .getByRole("region")
    .evaluateAll((els) => els.map((el) => el.getAttribute("aria-label") ?? ""));
  return names.join("|");
}

// dnd-kit uses an 8px pointer activation distance; move past it, then glide to
// the target in steps so the intermediate dragOver events fire.
async function dragTo(page: Page, source: Locator, target: Locator) {
  const s = await source.boundingBox();
  const t = await target.boundingBox();
  if (!s || !t) throw new Error("drag source or target has no bounding box");
  const sx = s.x + s.width / 2;
  const sy = s.y + s.height / 2;
  const tx = t.x + t.width / 2;
  const ty = t.y + t.height / 2;

  await page.mouse.move(sx, sy);
  await page.mouse.down();
  await page.mouse.move(sx + 16, sy, { steps: 5 });
  await page.mouse.move(tx, ty, { steps: 15 });
  await page.mouse.move(tx, ty, { steps: 3 });
  await page.mouse.up();
}

test("kanban happy path", async ({ page }) => {
  await test.step("create a project", async () => {
    await page.goto("/");
    await page.getByRole("button", { name: "Add project", exact: true }).click();
    const input = page.getByRole("textbox", {
      name: "New project name",
      exact: true,
    });
    await input.fill("Roadmap");
    await input.press("Enter");
    await expect(
      page.getByRole("link", { name: "Open Roadmap", exact: true }),
    ).toBeVisible();
  });

  await test.step("rename the project", async () => {
    await page
      .getByRole("button", { name: "Roadmap actions", exact: true })
      .click();
    await page.getByRole("menuitem", { name: "Rename", exact: true }).click();
    const input = page.getByRole("textbox", {
      name: "Project name",
      exact: true,
    });
    await input.fill("Roadmap 2026");
    await input.press("Enter");

    const renamed = page.getByRole("link", {
      name: "Open Roadmap 2026",
      exact: true,
    });
    await expect(renamed).toBeVisible();
    await page.reload();
    await expect(renamed).toBeVisible();
  });

  await test.step("create a board", async () => {
    await page
      .getByRole("link", { name: "Open Roadmap 2026", exact: true })
      .click();
    await page.getByRole("button", { name: "Add board", exact: true }).click();
    const input = page.getByRole("textbox", {
      name: "New board name",
      exact: true,
    });
    await input.fill("Sprint 1");
    await input.press("Enter");

    const board = page.getByRole("link", { name: "Open Sprint 1", exact: true });
    await expect(board).toBeVisible();
    await page.reload();
    await expect(board).toBeVisible();
  });

  await test.step("add three columns", async () => {
    await page.getByRole("link", { name: "Open Sprint 1", exact: true }).click();
    await expect(
      page.getByRole("heading", { name: "Sprint 1" }),
    ).toBeVisible();

    await page.getByRole("button", { name: "Add column", exact: true }).click();
    const input = page.getByRole("textbox", {
      name: "New column name",
      exact: true,
    });
    for (const name of ["To Do", "In Progress", "Done"]) {
      await input.fill(name);
      await input.press("Enter");
      await expect(region(page, name)).toBeVisible();
    }

    await page.reload();
    for (const name of ["To Do", "In Progress", "Done"]) {
      await expect(region(page, name)).toBeVisible();
    }
  });

  await test.step("reorder columns by drag", async () => {
    const before = await columnOrder(page);
    await dragTo(
      page,
      page.getByRole("button", { name: "Reorder To Do", exact: true }),
      region(page, "Done"),
    );
    await expect
      .poll(() => columnOrder(page), { timeout: 15_000 })
      .not.toBe(before);

    const after = await columnOrder(page);
    await page.reload();
    await expect.poll(() => columnOrder(page), { timeout: 15_000 }).toBe(after);
  });

  await test.step("add cards to a column", async () => {
    const toDo = region(page, "To Do");
    await toDo.getByRole("button", { name: "Add card", exact: true }).click();
    const input = toDo.getByRole("textbox", {
      name: "New card name",
      exact: true,
    });
    for (const title of ["Design", "Build"]) {
      await input.fill(title);
      await input.press("Enter");
      await expect(
        toDo.getByRole("button", { name: title, exact: true }),
      ).toBeVisible();
    }

    await page.reload();
    for (const title of ["Design", "Build"]) {
      await expect(
        region(page, "To Do").getByRole("button", { name: title, exact: true }),
      ).toBeVisible();
    }
  });

  await test.step("move a card across columns", async () => {
    await dragTo(
      page,
      region(page, "To Do").getByRole("button", { name: "Design", exact: true }),
      region(page, "Done"),
    );

    await expect(
      region(page, "Done").getByRole("button", { name: "Design", exact: true }),
    ).toBeVisible();
    await expect(
      region(page, "To Do").getByRole("button", { name: "Design", exact: true }),
    ).toHaveCount(0);

    await page.reload();
    await expect(
      region(page, "Done").getByRole("button", { name: "Design", exact: true }),
    ).toBeVisible();
  });

  await test.step("edit a card's description in the modal", async () => {
    await region(page, "Done")
      .getByRole("button", { name: "Design", exact: true })
      .click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    await dialog
      .getByLabel("Description", { exact: true })
      .fill("Ship it by Friday");
    await dialog
      .getByRole("button", { name: "Save description", exact: true })
      .click();
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();

    await page.reload();
    await region(page, "Done")
      .getByRole("button", { name: "Design", exact: true })
      .click();
    await expect(
      page.getByRole("dialog").getByLabel("Description", { exact: true }),
    ).toHaveValue("Ship it by Friday");
    await page.keyboard.press("Escape");
  });

  await test.step("delete a card", async () => {
    await region(page, "To Do")
      .getByRole("button", { name: "Build", exact: true })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Delete card", exact: true })
      .click();
    await page
      .getByRole("alertdialog")
      .getByRole("button", { name: "Delete", exact: true })
      .click();

    await page.reload();
    await expect(
      region(page, "To Do").getByRole("button", { name: "Build", exact: true }),
    ).toHaveCount(0);
  });

  await test.step("delete the board", async () => {
    await page.getByRole("link", { name: "Back to boards" }).click();
    const board = page.getByRole("link", { name: "Open Sprint 1", exact: true });
    await expect(board).toBeVisible();

    await page
      .getByRole("button", { name: "Sprint 1 actions", exact: true })
      .click();
    await page.getByRole("menuitem", { name: "Delete", exact: true }).click();
    await page
      .getByRole("alertdialog")
      .getByRole("button", { name: "Delete", exact: true })
      .click();

    await page.reload();
    await expect(board).toHaveCount(0);
  });

  await test.step("delete the project", async () => {
    await page.getByRole("link", { name: "Kanban", exact: true }).click();
    const project = page.getByRole("link", {
      name: "Open Roadmap 2026",
      exact: true,
    });
    await expect(project).toBeVisible();

    await page
      .getByRole("button", { name: "Roadmap 2026 actions", exact: true })
      .click();
    await page.getByRole("menuitem", { name: "Delete", exact: true }).click();
    await page
      .getByRole("alertdialog")
      .getByRole("button", { name: "Delete", exact: true })
      .click();

    await page.reload();
    await expect(project).toHaveCount(0);
  });
});
