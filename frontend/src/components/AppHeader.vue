<script setup>
const links = [
  { to: { name: 'home' }, label: 'Start' },
  { to: { name: 'schoolYears' }, label: 'Schuljahre' },
]
</script>

<template>
  <header class="border-b border-slate-200 bg-white">
    <nav class="mx-auto flex max-w-3xl items-center gap-1 px-4 py-2" aria-label="Hauptnavigation">
      <RouterLink
        :to="{ name: 'home' }"
        class="mr-3 text-xl font-semibold tracking-tight text-teal-700"
      >
        florg
      </RouterLink>
      <RouterLink
        v-for="link in links"
        :key="link.label"
        :to="link.to"
        class="rounded-lg px-3 py-2 text-sm font-medium text-slate-600"
        active-class="bg-teal-50 text-teal-800"
      >
        {{ link.label }}
      </RouterLink>
    </nav>
  </header>
</template>
