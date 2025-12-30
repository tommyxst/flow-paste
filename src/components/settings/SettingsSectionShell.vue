<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  title: string
  icon: Component
  iconClass: 'general' | 'ai' | 'actions'
  badge?: string
}>()
</script>

<template>
  <section class="settings-section" :aria-labelledby="`section-${title}`">
    <div class="section-header">
      <div class="section-icon" :class="iconClass">
        <component :is="icon" :size="16" aria-hidden="true" />
      </div>
      <h3 :id="`section-${title}`">{{ title }}</h3>
      <span v-if="badge" class="section-badge">{{ badge }}</span>
    </div>
    <div class="section-content">
      <slot />
    </div>
  </section>
</template>

<style src="./settings.shared.css"></style>
