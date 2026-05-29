<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useConfigStore } from '../stores/config'
import { useScriptStore } from '../stores/script'
import { useUIStore } from '../stores/ui'
import { Download, Plus, Upload, Trash2, Check, X, Gamepad, FileCode2, Minus } from '@lucide/vue'
import type { GameProfile } from '../types/config'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onUnmounted } from 'vue'
import type { Script } from '../types/script'

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

let unlistenOcrRegion: UnlistenFn | null = null

onMounted(async () => {
  store.fetchConfig()
  scriptStore.fetchScripts()

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
</script>

<template>
  <div class="config-panel">
    <div class="page-header">
      <h2>参数配置</h2>
    </div>

    <div class="config-sections">
      <!-- 通用设置 -->
      <section class="config-section">
        <h3>通用设置</h3>
        <div class="config-item">
          <label>开机自启动</label>
          <input type="checkbox" v-model="store.config.auto_start" />
        </div>
        <div class="config-item">
          <label>最小化到托盘</label>
          <input type="checkbox" v-model="store.config.minimize_to_tray" />
        </div>
        <div class="config-item">
          <label>日志级别</label>
          <select v-model="store.config.log_level" class="input">
            <option value="trace">Trace</option>
            <option value="debug">Debug</option>
            <option value="info">Info</option>
            <option value="warn">Warn</option>
            <option value="error">Error</option>
          </select>
        </div>
      </section>

      <!-- OCR 区域标定 -->
      <section class="config-section">
        <h3>OCR 自动化配置</h3>
        
        <div class="ocr-engine-selector">
          <div class="config-item">
            <label>OCR 识别引擎</label>
            <select v-model="store.config.ocr_engine" class="input">
              <option value="winocr">Windows 原生 (WinRT OCR)</option>
              <option value="paddleocr">外部 PaddleOCR (HTTP)</option>
            </select>
          </div>
          <div v-if="store.config.ocr_engine === 'paddleocr'" class="config-item">
            <label>PaddleOCR URL 地址</label>
            <input type="text" v-model="store.config.paddleocr_url" class="input url-input" placeholder="http://127.0.0.1:8050/ocr" />
          </div>
        </div>

        <div class="ocr-container-group">
          <!-- 区域列表 -->
          <div class="ocr-regions-list">
            <div 
              v-for="(region, idx) in (store.config.ocr_regions || [])" 
              :key="idx" 
              class="ocr-config-card"
            >
              <div class="ocr-status-group">
                <div class="ocr-label-group">
                  <span class="ocr-title">OCR 识别区 #{{ idx + 1 }}</span>
                  <span class="ocr-status-badge active">🎯 已标定</span>
                </div>
                <div class="ocr-coords">
                  <span class="coord-tag">X: {{ region.x }}</span>
                  <span class="coord-tag">Y: {{ region.y }}</span>
                  <span class="coord-tag">W: {{ region.w }}</span>
                  <span class="coord-tag">H: {{ region.h }}</span>
                </div>
                <div class="ocr-hint-code">
                  脚本调用: <code>ocr({{ idx + 1 }})</code>
                </div>
              </div>
              
              <div class="ocr-actions">
                <button 
                  class="action-btn text-btn delete-btn-simple" 
                  @click="clearOcrRegion(idx + 1)" 
                  title="清除此标定区域"
                >
                  删除
                </button>
                <button 
                  class="action-btn primary-btn ocr-btn-recal" 
                  @click="startOcrCalibration(idx + 1)" 
                  title="重新框选此标定区"
                >
                  重新标定
                </button>
              </div>
            </div>
          </div>

          <!-- 无配置占位图 -->
          <div v-if="!(store.config.ocr_regions && store.config.ocr_regions.length > 0)" class="ocr-empty-placeholder">
            <span class="ocr-empty-text">⚠️ 尚未标定任何 OCR 识别区</span>
            <span class="ocr-empty-desc">配置后即可在 Rhai 脚本中通过 <code>ocr()</code> 或 <code>ocr(序号)</code> 高效读取屏幕文字。</span>
          </div>

          <!-- 添加新区域按钮 -->
          <div class="ocr-add-container">
            <button 
              class="action-btn primary-btn ocr-btn" 
              @click="startOcrCalibration()" 
              title="添加一个新的屏幕框选识别区"
            >
              ➕ 添加标定区 (#{{ (store.config.ocr_regions || []).length + 1 }})
            </button>
          </div>
        </div>
      </section>

      <!-- Profile管理 -->
      <section class="config-section">
        <div class="section-header">
          <h3>Profile管理</h3>
          <div class="header-actions">
            <button class="action-btn text-btn" @click="triggerImport" title="导入Profile配置文件">
              <Upload :size="13" /> 导入
            </button>
            <button class="action-btn primary-btn" @click="openCreateModal" title="新建游戏手柄Profile配置">
              <Plus :size="13" /> 创建 Profile
            </button>
            <input type="file" ref="fileInput" @change="handleImport" accept=".json" class="hidden-input" />
          </div>
        </div>

        <div v-if="store.config.profiles.length === 0" class="empty-state">
          暂无 Profile，点击上方按钮创建
        </div>

        <div v-else class="profile-list">
          <div 
            v-for="profile in store.config.profiles" 
            :key="profile.id" 
            class="profile-card" 
          >
            <div class="profile-icon">
              <Gamepad :size="18" />
            </div>
            
            <div class="profile-main">
              <div class="profile-title">
                <h4>{{ profile.name }}</h4>
                <span class="controller-badge xbox360">Xbox 360</span>
              </div>
              <div class="profile-meta">
                <p class="profile-process">{{ profile.game_process }}</p>
                <span v-if="profile.scripts.length > 0" class="script-badge">
                  <FileCode2 :size="10" /> 脚本 ×{{ profile.scripts.length }}
                </span>
              </div>
            </div>

            <div class="profile-actions">
              <button 
                class="icon-btn script-btn" 
                title="管理绑定脚本" 
                @click="openScriptModal(profile)"
              >
                <FileCode2 :size="14" />
              </button>
              <button 
                class="icon-btn" 
                title="导出配置文件 (JSON)" 
                @click="handleExportProfile(profile)"
              >
                <Download :size="14" />
              </button>
              <button 
                class="icon-btn delete-btn" 
                title="删除此 Profile" 
                @click="handleDeleteProfile(profile.id)"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- 创建 Profile 弹窗 -->
    <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
      <div class="modal-card">
        <div class="modal-header">
          <h3>创建新 Profile</h3>
          <button class="close-btn" @click="showCreateModal = false">
            <X :size="16" />
          </button>
        </div>
        
        <div class="modal-body">
          <div class="form-group">
            <label>Profile 名称</label>
            <input 
              type="text" 
              v-model="profileForm.name" 
              placeholder="例如: 地平线5刷图配置" 
              class="form-input" 
              required 
            />
          </div>
          
          <div class="form-group">
            <label>游戏进程名称 (Game Process) <span class="label-optional">可选</span></label>
            <input 
              type="text" 
              v-model="profileForm.game_process" 
              placeholder="例如: ForzaHorizon5.exe (可空留)" 
              class="form-input" 
            />
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="btn-cancel" @click="showCreateModal = false">取消</button>
          <button class="btn-submit" @click="handleCreateProfile">创建</button>
        </div>
      </div>
    </div>

    <!-- 脚本绑定 Modal -->
    <div v-if="showScriptModal && editingProfile" class="modal-overlay" @click.self="closeScriptModal">
      <div class="modal-card modal-wide">
        <div class="modal-header">
          <div class="modal-title-group">
            <h3>管理脚本绑定</h3>
            <span class="modal-subtitle">{{ editingProfile.name }}</span>
          </div>
          <button class="close-btn" @click="closeScriptModal">
            <X :size="16" />
          </button>
        </div>

        <div class="modal-body script-modal-body">
          <!-- 已绑定脚本 -->
          <div class="script-section">
            <div class="script-section-label">
              <FileCode2 :size="13" />
              已绑定脚本
              <span class="count-badge">{{ boundScripts.length }}</span>
            </div>
            <div v-if="boundScripts.length === 0" class="script-empty">
              暂未绑定任何脚本
            </div>
            <div v-else class="script-list" @dragover.prevent>
              <div 
                v-for="(script, index) in boundScripts" 
                :key="script.id" 
                class="script-row bound"
                :class="{ 'drag-over': dragOverIndex === index }"
                draggable="true"
                @dragstart="handleDragStart(index, $event)"
                @dragover.prevent
                @dragenter.prevent="handleDragEnter(index)"
                @dragleave="handleDragLeave"
                @drop.prevent="handleDrop(index)"
              >
                <div class="script-row-left">
                  <span class="drag-handle" title="按住拖拽排序">☰</span>
                  <span class="script-row-name">{{ script.name }}</span>
                </div>
                
                <div class="script-row-right">
                  <div class="seq-sort-buttons">
                    <button 
                      class="seq-sort-btn" 
                      :disabled="index === 0" 
                      @click.stop="moveScript(index, -1)"
                      title="上移"
                    >
                      ▲
                    </button>
                    <button 
                      class="seq-sort-btn" 
                      :disabled="index === boundScripts.length - 1" 
                      @click.stop="moveScript(index, 1)"
                      title="下移"
                    >
                      ▼
                    </button>
                  </div>
                  <button class="script-row-btn remove-btn" @click="removeScriptFromProfile(script.id)" title="移除绑定">
                    <Minus :size="12" /> 移除
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="script-divider"></div>

          <!-- 可添加脚本 -->
          <div class="script-section">
            <div class="script-section-label">
              从脚本库添加
            </div>
            <div v-if="scriptStore.scripts.length === 0" class="script-empty">
              脚本库为空，请先在脚本编辑器中创建脚本
            </div>
            <div v-else-if="unboundScripts.length === 0" class="script-empty">
              所有脚本已全部绑定
            </div>
            <div v-else class="script-list">
              <div v-for="script in unboundScripts" :key="script.id" class="script-row unbound">
                <span class="script-row-name">{{ script.name }}</span>
                <button class="script-row-btn add-btn" @click="addScriptToProfile(script.id)" title="添加绑定">
                  <Plus :size="12" /> 添加
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn-submit" @click="closeScriptModal">完成</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-panel {
  padding: var(--space-lg);
  height: 100%;
  overflow-y: auto;
  background: var(--color-background);
}

.page-header {
  margin-bottom: var(--space-lg);
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.config-sections {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  max-width: 800px;
}

.config-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-lg);
  box-shadow: var(--shadow-sm);
}

.config-section h3 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: var(--space-md);
  color: var(--color-text);
  border-left: 3px solid var(--color-cta);
  padding-left: var(--space-xs);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
}

.section-header h3 {
  margin-bottom: 0;
}

.header-actions {
  display: flex;
  gap: var(--space-sm);
  align-items: center;
}

.hidden-input {
  display: none;
}

.config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
}

.config-item:last-child {
  border-bottom: none;
}

.config-item label {
  font-size: 13px;
  color: var(--color-text-muted);
}

.input {
  padding: var(--space-xs) var(--space-sm);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-size: 12px;
  outline: none;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.input:focus {
  border-color: var(--color-cta);
}

.empty-state {
  text-align: center;
  color: var(--color-text-dim);
  padding: var(--space-xl) var(--space-lg);
  font-size: 13px;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-md);
  margin-top: var(--space-md);
}

/* Profile List & Cards */
.profile-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  margin-top: var(--space-md);
}

.profile-card {
  display: flex;
  align-items: center;
  padding: var(--space-md);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition: all var(--transition-normal);
}

.profile-card:hover {
  transform: translateY(-1px);
  border-color: var(--color-text-dim);
  box-shadow: var(--shadow-md);
}

.profile-card.active {
  border-color: rgba(34, 197, 94, 0.4);
  background: rgba(34, 197, 94, 0.03);
}

.profile-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  margin-right: var(--space-md);
  flex-shrink: 0;
}

.profile-card.active .profile-icon {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
}

.profile-main {
  flex-grow: 1;
  min-width: 0;
}

.profile-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: 4px;
}

.profile-title h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.controller-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 10px;
  font-weight: 500;
}

.controller-badge.xbox360 {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.active-badge {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: 10px;
  background: rgba(34, 197, 94, 0.2);
  color: #22c55e;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  font-weight: 600;
}

.profile-meta {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.profile-process {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
  margin: 0;
}

.script-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 10px;
  color: var(--color-cta);
  background: rgba(99, 102, 241, 0.1);
  padding: 1px 6px;
  border-radius: 10px;
  font-weight: 500;
}

.profile-actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  flex-shrink: 0;
}

/* Action Buttons */
.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.primary-btn {
  background: var(--color-cta);
  color: white;
  border: none;
}

.primary-btn:hover {
  opacity: 0.9;
  transform: translateY(-0.5px);
}

.text-btn {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
}

.text-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-text);
}

.btn-activate {
  font-size: 11px;
  font-weight: 500;
  padding: 4px 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-activate:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
  border-color: var(--color-text-dim);
}

.btn-deactivate {
  font-size: 11px;
  font-weight: 500;
  padding: 4px 10px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-deactivate:hover {
  background: rgba(239, 68, 68, 0.2);
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-dim);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text);
}

.script-btn:hover {
  color: var(--color-cta) !important;
  background: rgba(99, 102, 241, 0.08) !important;
}

.delete-btn:hover {
  color: #ef4444 !important;
  background: rgba(239, 68, 68, 0.1) !important;
}

/* Modals */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(2, 6, 23, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease-out;
}

.modal-card {
  width: 100%;
  max-width: 440px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  overflow: hidden;
  animation: slideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-wide {
  max-width: 520px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
}

.modal-title-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.modal-header h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.modal-subtitle {
  font-size: 11px;
  color: var(--color-text-dim);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--color-text-dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  transition: all var(--transition-fast);
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text);
}

.modal-body {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

/* Script Modal */
.script-modal-body {
  gap: 0;
  padding: 0;
  max-height: 400px;
  overflow-y: auto;
}

.script-section {
  padding: var(--space-md) var(--space-lg);
}

.script-section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-sm);
}

.count-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: var(--color-cta);
  color: white;
  font-size: 10px;
  font-weight: 700;
  border-radius: 9px;
}

.script-divider {
  height: 1px;
  background: var(--color-border);
  margin: 0 var(--space-lg);
}

.script-empty {
  font-size: 12px;
  color: var(--color-text-dim);
  text-align: center;
  padding: var(--space-md);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-sm);
}

.script-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.script-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border);
  transition: all var(--transition-fast);
}

.script-row.bound {
  background: rgba(99, 102, 241, 0.05);
  border-color: rgba(99, 102, 241, 0.15);
}

.script-row.unbound {
  background: var(--color-surface-elevated);
}

.script-row.unbound:hover {
  border-color: var(--color-text-dim);
}

.script-row-name {
  font-size: 12px;
  color: var(--color-text);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.script-row-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 500;
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;
  margin-left: var(--space-sm);
}

.remove-btn {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.remove-btn:hover {
  background: rgba(239, 68, 68, 0.15);
}

.add-btn {
  background: rgba(99, 102, 241, 0.08);
  border: 1px solid rgba(99, 102, 241, 0.2);
  color: var(--color-cta);
}

.add-btn:hover {
  background: rgba(99, 102, 241, 0.15);
}

/* Form */
.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
}

.form-input, .form-select {
  padding: 8px 12px;
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-size: 13px;
  outline: none;
  transition: all var(--transition-fast);
}

.form-input:focus, .form-select:focus {
  border-color: var(--color-cta);
}

.form-input::placeholder {
  color: var(--color-text-dim);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  background: rgba(0, 0, 0, 0.15);
  border-top: 1px solid var(--color-border);
}

.btn-cancel {
  font-size: 12px;
  font-weight: 500;
  padding: 8px 16px;
  background: transparent;
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.03);
  color: var(--color-text);
}

.btn-submit {
  font-size: 12px;
  font-weight: 500;
  padding: 8px 16px;
  background: var(--color-cta);
  border: none;
  color: white;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-submit:hover {
  opacity: 0.9;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { transform: translateY(12px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

/* OCR Config styling */
.ocr-container-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.ocr-regions-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.ocr-config-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-md);
}

.ocr-status-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.ocr-label-group {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.ocr-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}

.ocr-status-badge {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 10px;
  font-weight: 600;
}

.ocr-status-badge.active {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.ocr-status-badge.inactive {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.ocr-coords {
  display: flex;
  gap: var(--space-xs);
  margin-top: 2px;
}

.coord-tag {
  font-size: 11px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  font-family: var(--font-heading);
}

.ocr-hint-code {
  font-size: 11px;
  color: var(--color-text-dim);
  margin-top: 2px;
}

.ocr-hint-code code {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-cta);
  padding: 1px 4px;
  border-radius: var(--radius-sm);
  font-family: var(--font-heading);
}

.ocr-empty-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-lg);
  background: rgba(255, 255, 255, 0.01);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-md);
  text-align: center;
  gap: var(--space-xs);
}

.ocr-empty-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.ocr-empty-desc {
  font-size: 11px;
  color: var(--color-text-dim);
}

.ocr-empty-desc code {
  color: var(--color-cta);
}

.ocr-add-container {
  display: flex;
  justify-content: flex-end;
}

.ocr-actions {
  display: flex;
  gap: var(--space-sm);
}

.delete-btn-simple {
  font-size: 11px;
  padding: 4px 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.delete-btn-simple:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444 !important;
  border-color: rgba(239, 68, 68, 0.2) !important;
}

.ocr-btn {
  background: var(--color-cta);
  border: none;
  font-size: 11px;
  font-weight: 500;
  padding: 6px 14px;
  border-radius: var(--radius-sm);
}

.ocr-btn:hover {
  background: #1ca84f;
}

.ocr-btn-recal {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border: 1px solid rgba(34, 197, 94, 0.2);
  font-size: 11px;
  font-weight: 500;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.ocr-btn-recal:hover {
  background: #22c55e;
  color: white;
}

.ocr-engine-selector {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: var(--space-md);
  border-bottom: 1px dashed var(--color-border);
  padding-bottom: var(--space-md);
}

.url-input {
  width: 280px;
  cursor: text;
}

.script-row.bound {
  cursor: grab;
  user-select: none;
  transition: background-color var(--transition-fast);
}

.script-row.bound:active {
  cursor: grabbing;
}

.script-row.bound.drag-over {
  border: 1px dashed var(--color-cta);
  background: rgba(34, 197, 94, 0.05) !important;
}

.script-row-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  pointer-events: none; /* 让点击穿透子节点以激活父级原生拖拽 */
}

.drag-handle {
  color: var(--color-text-dim);
  cursor: grab;
  font-size: 13px;
  user-select: none;
  padding-right: 4px;
}

.script-row-right {
  display: flex;
  align-items: center;
}

.seq-sort-buttons {
  display: flex;
  gap: 2px;
  margin-right: 4px;
}

.seq-sort-btn {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  color: var(--color-text-dim);
  border-radius: var(--radius-sm);
  padding: 1px 6px;
  font-size: 9px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.seq-sort-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-text);
  border-color: var(--color-text-dim);
}

.seq-sort-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
