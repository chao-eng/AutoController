<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useConfigStore } from '../stores/config'
import { useScriptStore } from '../stores/script'
import { useUIStore } from '../stores/ui'
import { Download, Plus, Upload, Trash2, Gamepad, FileCode2, Minus } from '@lucide/vue'
import type { GameProfile } from '../types/config'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onUnmounted } from 'vue'
import type { Script } from '../types/script'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import PageShell from '@/components/layout/PageShell.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import EmptyState from '@/components/layout/EmptyState.vue'

const store = useConfigStore()
const scriptStore = useScriptStore()
const uiStore = useUIStore()
const showCreateModal = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

// 脚本绑定 Modal
const showScriptModal = ref(false)
const editingProfileId = ref<string | null>(null)

const editingProfile = computed(() =>
  store.config.profiles.find(p => p.id === editingProfileId.value) ?? null
)

const boundScripts = computed(() => {
  if (!editingProfile.value) return []
  return editingProfile.value.scripts
    .map(id => scriptStore.scripts.find(s => s.id === id))
    .filter((s): s is Script => !!s)
})

const unboundScripts = computed(() => {
  if (!editingProfile.value) return []
  return scriptStore.scripts.filter(s => !editingProfile.value!.scripts.includes(s.id))
})

const draggedIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)

function handleDragStart(index: number, event: DragEvent) {
  draggedIndex.value = index
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', index.toString())
  }
}

function handleDragEnter(index: number) {
  dragOverIndex.value = index
}

function handleDragLeave() {
  dragOverIndex.value = null
}

async function handleDrop(targetIndex: number) {
  dragOverIndex.value = null
  if (draggedIndex.value === null || draggedIndex.value === targetIndex || !editingProfile.value) return
  
  const scripts = [...editingProfile.value.scripts]
  const [removed] = scripts.splice(draggedIndex.value, 1)
  scripts.splice(targetIndex, 0, removed)
  
  editingProfile.value.scripts = scripts
  await store.saveConfig()
  
  draggedIndex.value = null
}

async function moveScript(index: number, direction: number) {
  if (!editingProfile.value) return
  const targetIndex = index + direction
  if (targetIndex < 0 || targetIndex >= editingProfile.value.scripts.length) return
  
  const scripts = [...editingProfile.value.scripts]
  const [moved] = scripts.splice(index, 1)
  scripts.splice(targetIndex, 0, moved)
  
  editingProfile.value.scripts = scripts
  await store.saveConfig()
}

const profileForm = ref({
  name: '',
  game_process: ''
})

interface OcrTestResult {
  text: string
  engine: string
  profile: string
  x: number
  y: number
  w: number
  h: number
  scaled_w: number
  scaled_h: number
  scale: number
  capture_ms: number
  infer_ms: number
  total_ms: number
  inverted: boolean
}

const testingOcrIndex = ref<number | null>(null)
const ocrTestResults = ref<Record<number, OcrTestResult>>({})

let unlistenOcrRegion: UnlistenFn | null = null

onMounted(async () => {
  await store.fetchConfig()
  scriptStore.fetchScripts()
  void preheatCurrentOcr()

  try {
    unlistenOcrRegion = await listen<any>('ocr-region-saved', (event) => {
      const payload = event.payload
      if (payload && payload.regions) {
        store.config.ocr_regions = payload.regions
        if (payload.index === 1) {
          store.config.ocr_region = payload.region
        }
        uiStore.showToast(`🎯 OCR 识别区 #${payload.index} 标定成功！`, 'success')
      } else {
        store.config.ocr_region = event.payload
        uiStore.showToast('🎯 OCR 默认识别区标定成功！', 'success')
      }
    })
  } catch (e) {
    console.error('Failed to listen to ocr-region-saved event:', e)
  }
})

onUnmounted(() => {
  if (unlistenOcrRegion) {
    unlistenOcrRegion()
  }
})

function openCreateModal() {
  profileForm.value = {
    name: '',
    game_process: ''
  }
  showCreateModal.value = true
}

async function handleCreateProfile() {
  if (!profileForm.value.name.trim()) {
    uiStore.showToast('请填写 Profile 名称', 'warning')
    return
  }

  const newProfile: GameProfile = {
    id: 'p_' + Date.now().toString(36) + Math.random().toString(36).substring(2, 7),
    name: profileForm.value.name.trim(),
    game_process: profileForm.value.game_process.trim(),
    macros: [],
    scripts: []
  }

  store.config.profiles.push(newProfile)
  

  await store.saveConfig()
  showCreateModal.value = false
  uiStore.showToast('配置成功创建', 'success')
}



async function handleDeleteProfile(id: string) {
  const confirmed = await uiStore.showConfirm('确认删除', '确定要删除这个 Profile 吗？')
  if (!confirmed) {
    return
  }

  store.config.profiles = store.config.profiles.filter(p => p.id !== id)
  if (store.config.active_profile === id) {
    store.config.active_profile = null
  }

  await store.saveConfig()
  uiStore.showToast('配置删除成功', 'success')
}

// ── OCR 区域配置管理 ──────────────────────────────────────────
async function startOcrCalibration(index?: number) {
  try {
    // 传递给后端的 index 可以是 undefined (表示添加新标定区) 或具体的 1-based 序号
    await invoke('open_ocr_viewfinder', { index })
    uiStore.showToast('标定悬浮框已打开，请在屏幕上点击拖拽框选', 'info')
  } catch (err) {
    uiStore.showAlert('启动失败', `无法打开标定工具：${err}`)
  }
}

async function preheatCurrentOcr() {
  try {
    await invoke('preheat_ocr')
  } catch (err) {
    console.debug('OCR preheat skipped:', err)
  }
}

function ocrProfileName(profile?: string) {
  if (profile === 'fast') return '极速'
  if (profile === 'accurate') return '精细'
  return '平衡'
}

function getOcrTestResult(index: number) {
  return ocrTestResults.value[index]
}

async function testOcrRegion(index: number, region: { x: number; y: number; w: number; h: number }) {
  if (region.w <= 0 || region.h <= 0) {
    uiStore.showToast(`OCR 识别区 #${index} 坐标无效`, 'warning')
    return
  }

  testingOcrIndex.value = index
  try {
    const result = await invoke<OcrTestResult>('run_ocr_detailed', {
      x: region.x,
      y: region.y,
      w: region.w,
      h: region.h
    })
    ocrTestResults.value = {
      ...ocrTestResults.value,
      [index]: result
    }
    uiStore.showToast(`OCR 识别区 #${index} 测试完成`, 'success')
  } catch (err) {
    uiStore.showAlert('识别失败', `OCR 识别区 #${index} 测试失败：${err}`)
  } finally {
    testingOcrIndex.value = null
  }
}

async function clearOcrRegion(index: number) {
  const confirmed = await uiStore.showConfirm('清除配置', `确定要清除当前标定的 OCR 识别区 #${index} 吗？`)
  if (confirmed) {
    const regions = store.config.ocr_regions || []
    const vecIdx = index - 1
    if (vecIdx < regions.length) {
      regions.splice(vecIdx, 1)
      store.config.ocr_regions = [...regions]
      
      // 同步兼容单区域老字段
      if (index === 1) {
        store.config.ocr_region = regions.length > 0 ? regions[0] : null
      }
      
      await store.saveConfig()
      const nextResults = { ...ocrTestResults.value }
      delete nextResults[index]
      ocrTestResults.value = nextResults
      uiStore.showToast(`OCR 识别区 #${index} 配置已清除`, 'info')
    }
  }
}

// ── 脚本绑定管理 ────────────────────────────────────────────
function openScriptModal(profile: GameProfile) {
  editingProfileId.value = profile.id
  showScriptModal.value = true
}

function closeScriptModal() {
  showScriptModal.value = false
  editingProfileId.value = null
}

async function addScriptToProfile(scriptId: string) {
  const profile = editingProfile.value
  if (!profile) return
  if (!profile.scripts.includes(scriptId)) {
    profile.scripts.push(scriptId)
    await store.saveConfig()
  }
}

async function removeScriptFromProfile(scriptId: string) {
  const profile = editingProfile.value
  if (!profile) return
  profile.scripts = profile.scripts.filter(id => id !== scriptId)
  await store.saveConfig()
}

// ── 导出（含脚本数据）───────────────────────────────────────
async function handleExportProfile(profile: GameProfile) {
  try {
    // 批量获取绑定脚本的完整内容
    const scriptsData: Array<{ id: string; name: string; code: string }> = []
    for (const scriptId of profile.scripts) {
      try {
        const script = await invoke<Script>('script_get', { scriptId })
        scriptsData.push({ id: script.id, name: script.name, code: script.code })
      } catch {
        // 脚本可能已删除，跳过
      }
    }

    const exportPayload = {
      name: profile.name,
      game_process: profile.game_process,
      macros: profile.macros,
      scripts: profile.scripts,
      scripts_data: scriptsData
    }

    const fileName = `${profile.name.replace(/\s+/g, '_')}_profile.json`
    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(exportPayload, null, 2))
    const downloadAnchor = document.createElement('a')
    downloadAnchor.setAttribute("href", dataStr)
    downloadAnchor.setAttribute("download", fileName)
    document.body.appendChild(downloadAnchor)
    downloadAnchor.click()
    downloadAnchor.remove()

    const scriptCount = scriptsData.length
    const scriptNote = scriptCount > 0 ? `\n\n📎 已内嵌 ${scriptCount} 个脚本，导入时将自动还原。` : ''
    uiStore.showAlert(
      '导出成功',
      `文件已保存到系统默认下载目录\n\n📄 ${fileName}${scriptNote}`
    )
  } catch (err) {
    uiStore.showAlert('导出失败', `导出时发生错误：${err}`)
  }
}

// ── 导入（自动还原脚本）─────────────────────────────────────
function triggerImport() {
  fileInput.value?.click()
}

function handleImport(event: Event) {
  const input = event.target as HTMLInputElement
  if (!input.files || input.files.length === 0) return

  const file = input.files[0]
  const reader = new FileReader()
  reader.onload = async (e) => {
    try {
      const parsed = JSON.parse(e.target?.result as string)
      if (!parsed.name || !parsed.game_process) {
        uiStore.showAlert('导入失败', '导入失败：JSON 格式不正确，缺少必需的字段')
        return
      }

      // 还原脚本并建立旧 ID → 新 ID 的映射
      const idMap: Record<string, string> = {}
      let restoredCount = 0

      if (Array.isArray(parsed.scripts_data) && parsed.scripts_data.length > 0) {
        for (const sd of parsed.scripts_data) {
          if (!sd.name || typeof sd.code !== 'string') continue
          try {
            const created = await scriptStore.createScript(sd.name, sd.code)
            if (sd.id) idMap[sd.id] = created.id
            restoredCount++
          } catch {
            // 忽略单个脚本创建失败，继续
          }
        }
      }

      // 将旧 scripts 数组中的 ID 映射为新 ID（如果未在 scripts_data 中，保留原 ID）
      const remappedScripts: string[] = Array.isArray(parsed.scripts)
        ? parsed.scripts.map((id: string) => idMap[id] ?? id)
        : []

      const newProfile: GameProfile = {
        id: 'p_' + Date.now().toString(36) + Math.random().toString(36).substring(2, 7),
        name: parsed.name,
        game_process: parsed.game_process,
        macros: Array.isArray(parsed.macros) ? parsed.macros : [],
        scripts: remappedScripts
      }

      store.config.profiles.push(newProfile)


      await store.saveConfig()

      const msg = restoredCount > 0
        ? `成功导入 Profile: ${newProfile.name}（已还原 ${restoredCount} 个脚本）`
        : `成功导入 Profile: ${newProfile.name}`
      uiStore.showToast(msg, 'success')
    } catch (err) {
      uiStore.showAlert('解析失败', '解析文件失败，请确保导入的是有效的 Profile JSON 文件。')
    } finally {
      input.value = '' 
    }
  }
  reader.readAsText(file)
}

// ── 全局数据打包备份与还原 ────────────────────────────────────
const backupFileInput = ref<HTMLInputElement | null>(null)

async function exportBackup() {
  try {
    const data = await invoke<any>('export_backup_data')
    const now = new Date()
    const yyyy = now.getFullYear()
    const mm = String(now.getMonth() + 1).padStart(2, '0')
    const dd = String(now.getDate()).padStart(2, '0')
    const fileName = `autocontroller_backup_${yyyy}${mm}${dd}.json`
    
    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(data, null, 2))
    const downloadAnchor = document.createElement('a')
    downloadAnchor.setAttribute("href", dataStr)
    downloadAnchor.setAttribute("download", fileName)
    document.body.appendChild(downloadAnchor)
    downloadAnchor.click()
    downloadAnchor.remove()
    
    uiStore.showAlert(
      '备份成功',
      `全局数据已成功打包备份！\n\n📄 备份文件名: ${fileName}\n📁 保存位置: 已保存至您系统或浏览器的默认“下载”目录`
    )
  } catch (err) {
    uiStore.showAlert('备份失败', `无法导出备份数据：${err}`)
  }
}

function triggerImportBackup() {
  backupFileInput.value?.click()
}

function importBackup(event: Event) {
  const input = event.target as HTMLInputElement
  if (!input.files || input.files.length === 0) return

  const file = input.files[0]
  const reader = new FileReader()
  reader.onload = async (e) => {
    try {
      const parsed = JSON.parse(e.target?.result as string)
      if (!parsed.config && !parsed.macros && !parsed.scripts && !parsed.tasks) {
        uiStore.showAlert('导入失败', '导入失败：JSON 格式不正确，不是合法的备份文件')
        return
      }

      const confirmed = await uiStore.showConfirm(
        '确认恢复备份',
        '此操作将完全覆盖当前所有的配置、宏数据、脚本和定时任务，且无法撤销！是否确定导入并覆盖？'
      )
      if (!confirmed) {
        input.value = ''
        return
      }

      await invoke('import_backup_data', { backup: parsed })
      
      await store.fetchConfig()
      await scriptStore.fetchScripts()
      
      uiStore.showToast('🎉 全局数据已成功导入并还原！', 'success')
      
      setTimeout(() => {
        window.location.reload()
      }, 500)

    } catch (err) {
      uiStore.showAlert('导入失败', `解析或恢复备份文件失败。错误: ${err}`)
    } finally {
      input.value = ''
    }
  }
  reader.readAsText(file)
}
</script>

<template>
  <PageShell>
    <PageHeader
      title="参数配置"
      description="管理通用设置、OCR 标定、Profile 绑定和全局数据备份。"
    />

    <div class="flex flex-col gap-6">
      <!-- 通用设置 -->
      <Card>
        <CardHeader>
          <CardTitle class="text-sm border-l-[3px] border-primary pl-2">通用设置</CardTitle>
        </CardHeader>
        <CardContent class="space-y-3">
          <div class="flex items-center justify-between py-2 border-b border-border/30">
            <Label class="text-xs text-muted-foreground">开机自启动</Label>
            <Checkbox v-model:checked="store.config.auto_start" />
          </div>
          <div class="flex items-center justify-between py-2 border-b border-border/30">
            <Label class="text-xs text-muted-foreground">最小化到托盘</Label>
            <Checkbox v-model:checked="store.config.minimize_to_tray" />
          </div>
          <div class="flex items-center justify-between py-2">
            <Label class="text-xs text-muted-foreground">日志级别</Label>
            <Select v-model="store.config.log_level">
              <SelectTrigger class="w-[130px] h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="trace">Trace</SelectItem>
                <SelectItem value="debug">Debug</SelectItem>
                <SelectItem value="info">Info</SelectItem>
                <SelectItem value="warn">Warn</SelectItem>
                <SelectItem value="error">Error</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      <!-- 数据备份与恢复 -->
      <Card>
        <CardHeader>
          <CardTitle class="text-sm border-l-[3px] border-primary pl-2">数据备份与恢复</CardTitle>
        </CardHeader>
        <CardContent class="space-y-3">
          <CardDescription class="text-xs leading-relaxed">
            一键将程序的所有配置（含识别引擎）、宏录制数据、自定义脚本及定时任务打包备份为单 JSON 文件，或从备份文件还原全部数据资产。
          </CardDescription>
          <div class="flex items-center gap-2">
            <Button variant="default" size="sm" @click="exportBackup" title="导出打包备份数据">
              💾 备份全部数据
            </Button>
            <Button variant="outline" size="sm" @click="triggerImportBackup" title="导入备份并恢复数据">
              📂 导入恢复数据
            </Button>
            <input type="file" ref="backupFileInput" @change="importBackup" accept=".json" class="hidden" />
          </div>
        </CardContent>
      </Card>

      <!-- OCR 区域标定 -->
      <Card>
        <CardHeader>
          <CardTitle class="text-sm border-l-[3px] border-primary pl-2">OCR 自动化配置</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between py-2 border-b border-border/30">
            <Label class="text-xs text-muted-foreground">OCR 识别引擎</Label>
            <Select v-model="store.config.ocr_engine">
              <SelectTrigger class="w-[230px] h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="winocr">Windows 原生 (WinRT OCR)</SelectItem>
                <SelectItem value="paddleocr">内置 PaddleOCR (极速本地引擎)</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="flex items-center justify-between py-2 border-b border-border/30">
            <Label class="text-xs text-muted-foreground">识别策略</Label>
            <Select v-model="store.config.ocr_profile">
              <SelectTrigger class="w-[230px] h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="fast">极速 - 高频状态检测</SelectItem>
                <SelectItem value="balanced">平衡 - 默认推荐</SelectItem>
                <SelectItem value="accurate">精细 - 小字与弱对比</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div v-if="store.config.ocr_engine === 'paddleocr'" class="flex items-center gap-1.5 text-xs text-muted-foreground bg-primary/5 rounded-md px-3 py-2">
            <span>⚡</span>
            <span>已启用内置 PaddleOCR V4 本地推理引擎，启动后会后台预热模型。</span>
          </div>

          <div class="space-y-3">
            <div v-for="(region, idx) in (store.config.ocr_regions || [])" :key="idx" class="flex flex-col gap-3 bg-muted/30 border border-border rounded-lg p-3">
              <div class="flex items-center justify-between gap-3">
                <div class="space-y-1.5 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-semibold">OCR 识别区 #{{ idx + 1 }}</span>
                    <Badge variant="secondary" class="text-[10px] h-5">🎯 已标定</Badge>
                  </div>
                  <div class="flex flex-wrap gap-1.5">
                    <span class="text-[11px] bg-muted/50 border border-border px-1.5 py-0.5 rounded text-muted-foreground font-mono">X: {{ region.x }}</span>
                    <span class="text-[11px] bg-muted/50 border border-border px-1.5 py-0.5 rounded text-muted-foreground font-mono">Y: {{ region.y }}</span>
                    <span class="text-[11px] bg-muted/50 border border-border px-1.5 py-0.5 rounded text-muted-foreground font-mono">W: {{ region.w }}</span>
                    <span class="text-[11px] bg-muted/50 border border-border px-1.5 py-0.5 rounded text-muted-foreground font-mono">H: {{ region.h }}</span>
                  </div>
                  <div class="text-[11px] text-muted-foreground">
                    脚本调用: <code class="bg-muted/50 text-primary px-1 rounded font-mono text-[11px]">ocr({{ idx + 1 }})</code>
                  </div>
                </div>

                <div class="flex gap-2 shrink-0">
                  <Button variant="outline" size="sm" class="h-7 text-xs" :disabled="testingOcrIndex === idx + 1" @click="testOcrRegion(idx + 1, region)" title="对当前区域执行一次 OCR">
                    <FileCode2 :size="13" /> {{ testingOcrIndex === idx + 1 ? '识别中' : '测试识别' }}
                  </Button>
                  <Button variant="outline" size="sm" class="h-7 text-xs" @click="clearOcrRegion(idx + 1)" title="清除此标定区域">删除</Button>
                  <Button variant="default" size="sm" class="h-7 text-xs" @click="startOcrCalibration(idx + 1)" title="重新框选此标定区">重新标定</Button>
                </div>
              </div>

              <div v-if="getOcrTestResult(idx + 1)" class="rounded-md border border-border bg-background/60 p-2.5">
                <div class="flex flex-wrap items-center gap-2 mb-2 text-[11px] text-muted-foreground">
                  <Badge variant="secondary" class="text-[10px] h-5">
                    {{ getOcrTestResult(idx + 1)?.engine === 'winocr' ? 'WinRT' : 'PaddleOCR' }}
                  </Badge>
                  <span>策略: {{ ocrProfileName(getOcrTestResult(idx + 1)?.profile) }}</span>
                  <span>总耗时 {{ getOcrTestResult(idx + 1)?.total_ms }} ms</span>
                  <span>截图 {{ getOcrTestResult(idx + 1)?.capture_ms }} ms</span>
                  <span>推理 {{ getOcrTestResult(idx + 1)?.infer_ms }} ms</span>
                </div>
                <Textarea
                  readonly
                  :model-value="getOcrTestResult(idx + 1)?.text || '（未识别到文字）'"
                  class="min-h-16 resize-none bg-muted/20 font-mono text-xs"
                />
                <div class="mt-2 flex flex-wrap gap-2 text-[11px] text-muted-foreground">
                  <span>缩放 {{ getOcrTestResult(idx + 1)?.scale.toFixed(2) }}x</span>
                  <span>输入 {{ getOcrTestResult(idx + 1)?.scaled_w }} × {{ getOcrTestResult(idx + 1)?.scaled_h }} px</span>
                  <span v-if="getOcrTestResult(idx + 1)?.inverted">已启用暗底反色</span>
                </div>
              </div>
            </div>

            <div v-if="!(store.config.ocr_regions && store.config.ocr_regions.length > 0)" class="flex flex-col items-center gap-1 py-6 border border-dashed border-border rounded-lg text-center">
              <span class="text-sm font-semibold text-muted-foreground">⚠️ 尚未标定任何 OCR 识别区</span>
              <span class="text-[11px] text-muted-foreground/70">配置后即可在 Rhai 脚本中通过 <code class="bg-muted/50 text-primary px-1 rounded font-mono">ocr()</code> 或 <code class="bg-muted/50 text-primary px-1 rounded font-mono">ocr(序号)</code> 高效读取屏幕文字。</span>
            </div>

            <div class="flex justify-end">
              <Button variant="default" size="sm" @click="startOcrCalibration()" title="添加一个新的屏幕框选识别区">
                ➕ 添加标定区 (#{{ (store.config.ocr_regions || []).length + 1 }})
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Profile管理 -->
      <Card>
        <CardHeader class="flex flex-row items-center justify-between">
          <CardTitle class="text-sm border-l-[3px] border-primary pl-2">Profile管理</CardTitle>
          <div class="flex items-center gap-2">
            <Button variant="outline" size="sm" @click="triggerImport" title="导入Profile配置文件">
              <Upload :size="13" /> 导入
            </Button>
            <Button variant="default" size="sm" @click="openCreateModal" title="新建游戏手柄Profile配置">
              <Plus :size="13" /> 创建 Profile
            </Button>
            <input type="file" ref="fileInput" @change="handleImport" accept=".json" class="hidden" />
          </div>
        </CardHeader>
        <CardContent>
          <EmptyState
            v-if="store.config.profiles.length === 0"
            title="暂无 Profile"
            description="创建 Profile 后，可以绑定游戏进程和脚本，便于不同自动化场景快速切换。"
          />

          <div v-else class="space-y-1.5">
            <div v-for="profile in store.config.profiles" :key="profile.id" class="flex items-center gap-3 p-3 bg-muted/30 border border-border rounded-lg hover:-translate-y-0.5 hover:border-muted-foreground/50 hover:shadow-md transition-all">
              <div class="flex items-center justify-center size-9 bg-muted/30 rounded text-muted-foreground shrink-0">
                <Gamepad :size="18" />
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-0.5">
                  <h4 class="text-sm font-semibold m-0">{{ profile.name }}</h4>
                  <Badge variant="secondary" class="text-[10px] h-5 text-green-600 bg-green-500/15">Xbox 360</Badge>
                </div>
                <div class="flex items-center gap-2">
                  <p class="text-[11px] text-muted-foreground/60 font-mono m-0">{{ profile.game_process }}</p>
                  <span v-if="profile.scripts.length > 0" class="inline-flex items-center gap-1 text-[10px] text-primary bg-primary/10 px-1.5 py-0.5 rounded-full font-medium">
                    <FileCode2 :size="10" /> 脚本 ×{{ profile.scripts.length }}
                  </span>
                </div>
              </div>

              <div class="flex items-center gap-0.5 shrink-0">
                <Button variant="ghost" size="icon" class="size-7" title="管理绑定脚本" @click="openScriptModal(profile)">
                  <FileCode2 :size="14" />
                </Button>
                <Button variant="ghost" size="icon" class="size-7" title="导出配置文件 (JSON)" @click="handleExportProfile(profile)">
                  <Download :size="14" />
                </Button>
                <Button variant="ghost" size="icon" class="size-7 text-destructive hover:text-destructive hover:bg-destructive/10" title="删除此 Profile" @click="handleDeleteProfile(profile.id)">
                  <Trash2 :size="14" />
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- 创建 Profile 弹窗 -->
    <Dialog :open="showCreateModal" @update:open="showCreateModal = $event">
      <DialogContent class="sm:max-w-[440px]">
        <DialogHeader>
          <DialogTitle>创建新 Profile</DialogTitle>
          <DialogDescription class="sr-only">填写 Profile 名称和游戏进程信息</DialogDescription>
        </DialogHeader>
        <div class="space-y-4">
          <div class="space-y-2">
            <Label class="text-xs font-medium text-muted-foreground">Profile 名称</Label>
            <Input v-model="profileForm.name" placeholder="例如: 地平线5刷图配置" required />
          </div>
          <div class="space-y-2">
            <Label class="text-xs font-medium text-muted-foreground">
              游戏进程名称 (Game Process) <span class="text-muted-foreground/50">可选</span>
            </Label>
            <Input v-model="profileForm.game_process" placeholder="例如: ForzaHorizon5.exe (可空留)" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" size="sm" @click="showCreateModal = false">取消</Button>
          <Button variant="default" size="sm" @click="handleCreateProfile">创建</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 脚本绑定 Modal -->
    <Dialog :open="showScriptModal && !!editingProfile" @update:open="(v) => { if (!v) closeScriptModal() }">
      <DialogContent class="sm:max-w-[520px] max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>管理脚本绑定</DialogTitle>
          <DialogDescription class="sr-only">{{ editingProfile?.name || '脚本绑定管理' }}</DialogDescription>
          <template #description>
            <span class="text-[11px] text-muted-foreground">{{ editingProfile?.name }}</span>
          </template>
        </DialogHeader>
        
        <div class="flex-1 overflow-y-auto -mx-6 px-6">
          <!-- 已绑定脚本 -->
          <div class="py-3">
            <div class="flex items-center gap-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-2">
              <FileCode2 :size="13" />
              已绑定脚本
              <span class="inline-flex items-center justify-center min-w-[18px] h-[18px] px-1 bg-primary text-white text-[10px] font-bold rounded-full">{{ boundScripts.length }}</span>
            </div>
            <div v-if="boundScripts.length === 0" class="text-xs text-muted-foreground text-center py-4 border border-dashed border-border rounded">
              暂未绑定任何脚本
            </div>
            <div v-else class="space-y-1" @dragover.prevent>
              <div v-for="(script, index) in boundScripts" :key="script.id"
                class="flex items-center justify-between px-2.5 py-2 rounded border bg-primary/5 border-primary/20"
                :class="{ 'ring-2 ring-primary': dragOverIndex === index }"
                draggable="true"
                @dragstart="handleDragStart(index, $event)"
                @dragover.prevent
                @dragenter.prevent="handleDragEnter(index)"
                @dragleave="handleDragLeave"
                @drop.prevent="handleDrop(index)"
              >
                <div class="flex items-center gap-2 flex-1 min-w-0">
                  <span class="cursor-grab text-muted-foreground/50 text-sm" title="按住拖拽排序">☰</span>
                  <span class="text-xs font-medium truncate">{{ script.name }}</span>
                </div>
                <div class="flex items-center gap-1.5 shrink-0">
                  <Button variant="ghost" size="icon" class="size-6 text-[10px]" :disabled="index === 0" @click.stop="moveScript(index, -1)" title="上移">▲</Button>
                  <Button variant="ghost" size="icon" class="size-6 text-[10px]" :disabled="index === boundScripts.length - 1" @click.stop="moveScript(index, 1)" title="下移">▼</Button>
                  <Button variant="outline" size="sm" class="h-7 text-[11px] text-destructive border-destructive/30 bg-destructive/5 hover:bg-destructive/15" @click="removeScriptFromProfile(script.id)" title="移除绑定">
                    <Minus :size="12" /> 移除
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <Separator />

          <!-- 可添加脚本 -->
          <div class="py-3">
            <div class="flex items-center gap-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider mb-2">从脚本库添加</div>
            <div v-if="scriptStore.scripts.length === 0" class="text-xs text-muted-foreground text-center py-4 border border-dashed border-border rounded">
              脚本库为空，请先在脚本编辑器中创建脚本
            </div>
            <div v-else-if="unboundScripts.length === 0" class="text-xs text-muted-foreground text-center py-4 border border-dashed border-border rounded">
              所有脚本已全部绑定
            </div>
            <div v-else class="space-y-1">
              <div v-for="script in unboundScripts" :key="script.id" class="flex items-center justify-between px-2.5 py-2 rounded border border-border bg-muted/30 hover:border-muted-foreground/50 transition-colors">
                <span class="text-xs font-medium truncate">{{ script.name }}</span>
                <Button variant="outline" size="sm" class="h-7 text-[11px] text-primary border-primary/30 bg-primary/5 hover:bg-primary/15" @click="addScriptToProfile(script.id)" title="添加绑定">
                  <Plus :size="12" /> 添加
                </Button>
              </div>
            </div>
          </div>
        </div>

        <DialogFooter class="border-t border-border pt-4 -mx-6 px-6">
          <Button variant="default" size="sm" @click="closeScriptModal">完成</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageShell>
</template>
