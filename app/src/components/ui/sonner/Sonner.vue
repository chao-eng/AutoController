<script lang="ts" setup>
import type { ToasterProps } from 'vue-sonner'

import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
  XIcon,
} from '@lucide/vue'
import { Toaster as Sonner } from 'vue-sonner'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<ToasterProps>(), {
  closeButtonPosition: 'top-right',
  expand: true,
  gap: 4,
  toastOptions: () => ({
    classes: {
      toast: 'ac-toast',
      content: 'ac-toast__content',
      title: 'ac-toast__title',
      description: 'ac-toast__description',
      icon: 'ac-toast__icon',
      closeButton: 'ac-toast__close',
      actionButton: 'ac-toast__action',
      cancelButton: 'ac-toast__cancel',
    },
  }),
})
</script>

<template>
  <Sonner
    :class="cn('toaster group', props.class)"
    :style="{
      '--normal-bg': 'var(--popover)',
      '--normal-text': 'var(--popover-foreground)',
      '--normal-border': 'var(--border)',
      '--border-radius': 'var(--radius)',
      '--gray2': 'var(--muted)',
      '--gray3': 'var(--border)',
      '--gray4': 'var(--border)',
      '--gray5': 'var(--border)',
      '--gray12': 'var(--popover-foreground)',
    }"
    v-bind="props"
  >
    <template #success-icon>
      <CircleCheckIcon class="size-4" />
    </template>
    <template #info-icon>
      <InfoIcon class="size-4" />
    </template>
    <template #warning-icon>
      <TriangleAlertIcon class="size-4" />
    </template>
    <template #error-icon>
      <OctagonXIcon class="size-4" />
    </template>
    <template #loading-icon>
      <div>
        <Loader2Icon class="size-4 animate-spin" />
      </div>
    </template>
    <template #close-icon>
      <XIcon class="size-3.5" />
    </template>
  </Sonner>
</template>

<style>
[data-sonner-toaster].toaster {
  --width: min(336px, calc(100vw - 32px));
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-family: var(--font-sans);
}

[data-sonner-toaster].toaster [data-sonner-toast] {
  position: relative;
  top: auto;
  right: auto;
  bottom: auto;
  left: auto;
  transform: translateY(0) !important;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-expanded='true']::after {
  display: none;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-removed='true'] {
  opacity: 0;
  transform: translateX(10px) !important;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] {
  --toast-tint: var(--status-neutral-bg);
  --toast-border: var(--border);
  --toast-accent: var(--status-neutral-text);
  --toast-panel: color-mix(in oklch, var(--popover) 90%, transparent);
  position: relative;
  min-height: 46px;
  align-items: flex-start;
  gap: 9px;
  overflow: hidden;
  padding: 9px 38px 9px 13px;
  border-color: var(--toast-border);
  border-radius: var(--radius);
  background:
    linear-gradient(90deg, color-mix(in oklch, var(--toast-tint) 58%, transparent), transparent 48%),
    var(--toast-panel);
  color: var(--popover-foreground);
  box-shadow:
    inset 2px 0 0 var(--toast-accent),
    0 8px 22px oklch(0 0 0 / 0.08),
    inset 0 1px 0 oklch(1 0 0 / 0.45);
  backdrop-filter: blur(16px) saturate(1.05);
  -webkit-backdrop-filter: blur(16px) saturate(1.05);
}

.dark [data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] {
  --toast-panel: color-mix(in oklch, var(--popover) 86%, transparent);
  box-shadow:
    inset 2px 0 0 var(--toast-accent),
    0 14px 34px oklch(0 0 0 / 0.34),
    inset 0 1px 0 oklch(1 0 0 / 0.08);
}

[data-sonner-toaster].toaster [data-sonner-toast][data-type='success'] {
  --toast-tint: var(--status-success-bg);
  --toast-accent: var(--status-success-text);
}

[data-sonner-toaster].toaster [data-sonner-toast][data-type='info'] {
  --toast-tint: var(--status-info-bg);
  --toast-accent: var(--status-info-text);
}

[data-sonner-toaster].toaster [data-sonner-toast][data-type='warning'] {
  --toast-tint: var(--status-warning-bg);
  --toast-accent: var(--status-warning-text);
}

[data-sonner-toaster].toaster [data-sonner-toast][data-type='error'] {
  --toast-tint: var(--status-danger-bg);
  --toast-accent: var(--status-danger-text);
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] [data-icon].ac-toast__icon {
  width: 21px;
  height: 21px;
  margin: 0;
  border: 1px solid color-mix(in oklch, var(--toast-accent) 24%, transparent);
  border-radius: min(var(--radius-sm), 7px);
  background: color-mix(in oklch, var(--toast-tint) 78%, transparent);
  color: var(--toast-accent);
  justify-content: center;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] [data-icon].ac-toast__icon svg {
  width: 14px;
  height: 14px;
  margin: 0;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__content {
  min-width: 0;
  flex: 1;
  gap: 1px;
  padding-top: 0;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__title {
  color: var(--foreground);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0;
  line-height: 18px;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__description {
  color: var(--muted-foreground);
  font-size: 11px;
  line-height: 16px;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__close[data-close-button] {
  right: 7px;
  top: 6px;
  bottom: unset;
  left: unset;
  width: 19px;
  height: 19px;
  border-color: transparent;
  border-radius: min(var(--radius-sm), 7px);
  background: transparent;
  color: var(--muted-foreground);
  opacity: 0.72;
  transform: none;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__close[data-close-button]:hover {
  border-color: color-mix(in oklch, var(--toast-accent) 24%, transparent);
  background: color-mix(in oklch, var(--toast-tint) 82%, transparent);
  color: var(--toast-accent);
  opacity: 1;
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__close[data-close-button]:focus-visible {
  box-shadow: 0 0 0 3px color-mix(in oklch, var(--toast-accent) 22%, transparent);
}

[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__action,
[data-sonner-toaster].toaster [data-sonner-toast][data-styled='true'] .ac-toast__cancel {
  height: 26px;
  border-radius: min(var(--radius-sm), 7px);
  font-size: 11px;
}
</style>
