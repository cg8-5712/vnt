<script setup lang="ts">
import { computed } from 'vue'

import type { VntStatus } from '@/types/vnt'
import { statusLabel } from '@/utils/format'

const props = defineProps<{
  status: VntStatus
}>()

const badgeClass = computed(() => `status-badge is-${props.status}`)
</script>

<template>
  <span :class="badgeClass">
    <i />
    {{ statusLabel(status) }}
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  padding: 0.55rem 0.85rem;
  font-size: 0.92rem;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.04);
}

.status-badge i {
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 999px;
  display: inline-block;
}

.is-running {
  color: #92f3cb;
}

.is-running i {
  background: #42c79a;
  box-shadow: 0 0 1rem rgba(66, 199, 154, 0.75);
}

.is-starting {
  color: #ffd678;
}

.is-starting i {
  background: #f6b938;
  box-shadow: 0 0 1rem rgba(246, 185, 56, 0.72);
}

.is-stopped {
  color: #ff8f8f;
}

.is-stopped i {
  background: #ef5a5a;
  box-shadow: 0 0 1rem rgba(239, 90, 90, 0.72);
}
</style>
