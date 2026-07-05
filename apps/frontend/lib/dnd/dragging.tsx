"use client";

import { createContext, useContext } from "react";

const DraggingContext = createContext(false);

export const DraggingProvider = DraggingContext.Provider;

export function useIsDragging() {
  return useContext(DraggingContext);
}
