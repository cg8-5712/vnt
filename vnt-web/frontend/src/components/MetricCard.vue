<script setup lang="ts">
defineProps<{
  label: string
  value: string | number
  hint?: string
}>()
</script>

<template>
  <article class="metric-card">
    <span class="metric-label">{{ label }}</span>
    <strong class="metric-value">{{ value }}</strong>
    <span v-if="hint" class="metric-hint">{{ hint }}</span>
  </article>
</template>

<style scoped>
.metric-card {
  display: flex;
  min-height: 7rem;
  flex-direction: column;
  justify-content: space-between;
  gap: 0.8rem;
  border: 1px solid var(--border-strong);
  border-radius: 1.25rem;
  background:
    linear-gradient(145deg, rgba(13, 28, 48, 0.9), rgba(8, 19, 33, 0.88)),
    var(--panel);
  padding: 1.15rem 1.2rem;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.metric-label {
  color: var(--text-soft);
  font-size: 0.85rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.metric-value {
  font-size: clamp(1.4rem, 4vw, 2rem);
  line-height: 1;
}

.metric-hint {
  color: var(--text-soft);
  font-size: 0.92rem;
}
</style>
