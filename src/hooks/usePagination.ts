import { useState, useCallback, useMemo } from "react";

interface UsePaginationOptions {
  totalItems: number;
  pageSize?: number;
  initialPage?: number;
}

/**
 * Generic pagination hook
 */
export function usePagination(options: UsePaginationOptions) {
  const { totalItems, pageSize = 100, initialPage = 1 } = options;
  const [currentPage, setCurrentPage] = useState(initialPage);

  const totalPages = useMemo(
    () => Math.max(1, Math.ceil(totalItems / pageSize)),
    [totalItems, pageSize]
  );

  const offset = useMemo(
    () => (currentPage - 1) * pageSize,
    [currentPage, pageSize]
  );

  const hasNextPage = currentPage < totalPages;
  const hasPrevPage = currentPage > 1;

  const goToPage = useCallback(
    (page: number) => {
      setCurrentPage(Math.max(1, Math.min(page, totalPages)));
    },
    [totalPages]
  );

  const nextPage = useCallback(() => {
    if (hasNextPage) setCurrentPage((p) => p + 1);
  }, [hasNextPage]);

  const prevPage = useCallback(() => {
    if (hasPrevPage) setCurrentPage((p) => p - 1);
  }, [hasPrevPage]);

  const reset = useCallback(() => {
    setCurrentPage(1);
  }, []);

  return {
    currentPage,
    totalPages,
    offset,
    pageSize,
    hasNextPage,
    hasPrevPage,
    goToPage,
    nextPage,
    prevPage,
    reset,
  };
}
