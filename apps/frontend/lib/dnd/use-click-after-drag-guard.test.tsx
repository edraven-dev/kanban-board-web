import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useClickAfterDragGuard } from "./use-click-after-drag-guard";

function clickEvent() {
  return {
    preventDefault: vi.fn(),
    stopPropagation: vi.fn(),
  } as unknown as React.MouseEvent;
}

describe("useClickAfterDragGuard", () => {
  it("swallows the click fired after a drag", () => {
    const { result, rerender } = renderHook(
      ({ dragging }) => useClickAfterDragGuard(dragging),
      { initialProps: { dragging: false } },
    );

    rerender({ dragging: true });

    const event = clickEvent();
    result.current.onClickCapture(event);
    expect(event.preventDefault).toHaveBeenCalled();
    expect(event.stopPropagation).toHaveBeenCalled();
  });

  it("lets a normal click through (reset on pointer down)", () => {
    const { result, rerender } = renderHook(
      ({ dragging }) => useClickAfterDragGuard(dragging),
      { initialProps: { dragging: false } },
    );

    rerender({ dragging: true });
    result.current.onPointerDownCapture();

    const event = clickEvent();
    result.current.onClickCapture(event);
    expect(event.preventDefault).not.toHaveBeenCalled();
  });
});
