import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { usePagination } from "../../hooks/usePagination";

describe("usePagination", () => {
  it("should initialize with correct defaults", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 250 }));
    expect(result.current.currentPage).toBe(1);
    expect(result.current.totalPages).toBe(3); // 250/100 = 3
    expect(result.current.offset).toBe(0);
    expect(result.current.hasNextPage).toBe(true);
    expect(result.current.hasPrevPage).toBe(false);
  });

  it("should handle custom page size", () => {
    const { result } = renderHook(() =>
      usePagination({ totalItems: 250, pageSize: 50 })
    );
    expect(result.current.totalPages).toBe(5);
  });

  it("should navigate to next page", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 250 }));
    act(() => result.current.nextPage());
    expect(result.current.currentPage).toBe(2);
    expect(result.current.offset).toBe(100);
  });

  it("should navigate to previous page", () => {
    const { result } = renderHook(() =>
      usePagination({ totalItems: 250, initialPage: 2 })
    );
    act(() => result.current.prevPage());
    expect(result.current.currentPage).toBe(1);
  });

  it("should not go below page 1", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 250 }));
    act(() => result.current.prevPage());
    expect(result.current.currentPage).toBe(1);
  });

  it("should not go above last page", () => {
    const { result } = renderHook(() =>
      usePagination({ totalItems: 100, initialPage: 1 })
    );
    act(() => result.current.nextPage());
    expect(result.current.currentPage).toBe(1); // only 1 page
  });

  it("should go to specific page", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 500 }));
    act(() => result.current.goToPage(3));
    expect(result.current.currentPage).toBe(3);
    expect(result.current.offset).toBe(200);
  });

  it("should clamp page to valid range", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 250 }));
    act(() => result.current.goToPage(100));
    expect(result.current.currentPage).toBe(3); // max page
    act(() => result.current.goToPage(-1));
    expect(result.current.currentPage).toBe(1); // min page
  });

  it("should reset to page 1", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 250 }));
    act(() => result.current.goToPage(3));
    act(() => result.current.reset());
    expect(result.current.currentPage).toBe(1);
  });

  it("should handle zero items", () => {
    const { result } = renderHook(() => usePagination({ totalItems: 0 }));
    expect(result.current.totalPages).toBe(1);
    expect(result.current.hasNextPage).toBe(false);
  });
});
