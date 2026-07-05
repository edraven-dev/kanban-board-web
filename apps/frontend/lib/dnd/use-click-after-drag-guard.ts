"use client";

import { type MouseEvent as ReactMouseEvent, useEffect, useRef } from "react";

/**
 * Guards a draggable-and-clickable element: swallows the click the browser
 * fires after a drag (so dragging doesn't also navigate/open). Spread the
 * returned handlers on the element and pass `isDragging` from `useSortable`.
 */
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
