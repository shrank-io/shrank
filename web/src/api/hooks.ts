import {
  useQuery,
  useMutation,
  useQueryClient,
  useInfiniteQuery,
  type QueryKey,
} from "@tanstack/react-query";
import * as api from "./client";
import type { DocumentUpdate } from "./types";

// --- Documents ---

export function useDocuments(params: {
  limit?: number;
  status?: string;
  sender?: string;
  type?: string;
  tag?: string;
  sort?: string;
}) {
  return useInfiniteQuery({
    queryKey: ["documents", params] as QueryKey,
    queryFn: ({ pageParam = 0 }) =>
      api.listDocuments({ ...params, limit: params.limit ?? 30, offset: pageParam }),
    initialPageParam: 0,
    getNextPageParam: (last, allPages) => {
      const loaded = allPages.flatMap((p) => p.documents).length;
      return loaded < last.total ? loaded : undefined;
    },
  });
}

export function useDocument(id: string | undefined) {
  return useQuery({
    queryKey: ["document", id],
    queryFn: () => api.getDocument(id!),
    enabled: !!id,
  });
}

export function useUpdateDocument() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: DocumentUpdate }) =>
      api.updateDocument(id, data),
    onSuccess: (doc) => {
      qc.setQueryData(["document", doc.id], doc);
      qc.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useDeleteDocument() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: api.deleteDocument,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useReprocessDocument() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: api.reprocessDocument,
    onSuccess: (doc) => {
      qc.setQueryData(["document", doc.id], doc);
      qc.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}

export function useUploadDocument() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ file, onProgress }: { file: File; onProgress?: (pct: number) => void }) =>
      api.uploadDocument(file, onProgress),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["documents"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
    },
  });
}

// --- Search ---

export function useSearch(q: string, limit?: number) {
  return useInfiniteQuery({
    queryKey: ["search", q, limit] as QueryKey,
    queryFn: ({ pageParam = 0 }) =>
      api.search({ q, limit: limit ?? 20, offset: pageParam }),
    initialPageParam: 0,
    getNextPageParam: (last) => {
      const next = (last.results.length > 0 ? last.results.length : 0);
      return next < last.total ? next : undefined;
    },
    enabled: q.length > 0,
  });
}

// --- Graph ---

export function useRelatedDocuments(id: string | undefined, depth?: number) {
  return useQuery({
    queryKey: ["related", id, depth],
    queryFn: () => api.getRelatedDocuments(id!, depth),
    enabled: !!id,
  });
}

// --- Stats ---

export function useStats() {
  return useQuery({
    queryKey: ["stats"],
    queryFn: api.getStats,
    refetchInterval: 30_000,
  });
}

// --- Images ---

/** Fetch an image with auth and return an object URL. Revoke on unmount via React Query GC. */
export function useAuthImage(url: string | undefined) {
  return useQuery({
    queryKey: ["image", url],
    queryFn: () => api.fetchImageAsObjectUrl(url!),
    enabled: !!url,
    staleTime: Infinity,
    gcTime: 5 * 60_000,
  });
}
