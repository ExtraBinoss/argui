export function usePageSeo(title: MaybeRefOrGetter<string>, description: MaybeRefOrGetter<string>) {
  const route = useRoute()
  const config = useRuntimeConfig()
  const origin = config.public.siteUrl.replace(/\/$/, '')
  const canonical = computed(() =>
    origin ? `${origin}${route.path === '/' ? '/' : route.path.replace(/\/$/, '')}` : undefined,
  )
  useSeoMeta({
    title,
    description,
    ogTitle: title,
    ogDescription: description,
    ogType: 'website',
    ogSiteName: 'Argui',
    ogLocale: 'en',
    ogUrl: canonical,
    ogImage: origin ? `${origin}/social.png` : undefined,
    twitterCard: 'summary_large_image',
  })
  useHead(() => ({ link: canonical.value ? [{ rel: 'canonical', href: canonical.value }] : [] }))
}
