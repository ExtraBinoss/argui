export function usePublicAsset() {
  const base = useRuntimeConfig().app.baseURL
  return (path: string) => `${base.replace(/\/$/, '')}/${path.replace(/^\//, '')}`
}
