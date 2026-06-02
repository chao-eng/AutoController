<script setup lang="ts">
import { HelpCircle } from '@lucide/vue'
import { ref } from 'vue'
import HelpModal from '../HelpModal.vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Button } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

const showHelp = ref(false)

async function openGithub() {
  try {
    await openUrl('https://github.com/chao-eng/AutoController')
  } catch (err) {
    console.error('无法在系统浏览器中打开网页:', err)
  }
}
</script>

<template>
  <header class="flex h-9 items-center justify-between border-b border-border bg-background px-3 flex-shrink-0" data-tauri-drag-region>
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <span class="text-xs font-semibold tracking-wide text-muted-foreground">手柄自动化控制台</span>
    </div>
    <div class="flex items-center gap-2">
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon-sm" @click="openGithub">
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
              </svg>
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>项目 GitHub 源码</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon-sm" @click="showHelp = true">
              <HelpCircle :size="14" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>使用说明</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
    <HelpModal v-if="showHelp" @close="showHelp = false" />
  </header>
</template>