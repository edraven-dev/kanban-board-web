"use client";

import { type MouseEvent as ReactMouseEvent, useEffect, useRef } from "react";

/** Swallows the click the browser fires after a drag, so dragging doesn't also navigate/open. */
export function useClickAfterDragGuard(isDragging: boolean) {
  const draggedRef = useRef(false);

  useEffect(() => {
    if (isDragging) draggedRef.current = true;
  }, [isDragging]);

  return {
    onPointerDownCapture: () => {
      draggedRef.current = false;
    },
    onClickCapture: (event: ReactMouseEvent) => {
      if (draggedRef.current) {
        event.preventDefault();
        event.stopPropagation();
        draggedRef.current = false;
      }
    },
  };
}
