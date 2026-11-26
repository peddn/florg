<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'

import AppHeader from './components/AppHeader.vue'
import StatusBar from './components/StatusBar.vue'

const route = useRoute()
// The login pages stand alone. `route.name` is unset until the first
// navigation has finished.
const showHeader = computed(() => route.name && !route.meta.public)
</script>

<template>
  <!-- The bottom padding keeps content clear of the fixed status bar. -->
  <div class="min-h-dvh bg-slate-50 pb-8 text-slate-900">
    <AppHeader v-if="showHeader" />
    <RouterView />
    <StatusBar />
  </div>
</template>
