<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useConfigStore } from '../stores/config'
import { Download, Plus, Upload, Trash2, Check, X, Gamepad } from '@lucide/vue'
import type { GameProfile } from '../types/config'

const store = useConfigStore()
const showCreateModal = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const profileForm = ref({
  name: '',
  game_process: '',
  controller_type: 'xbox360' as 'xbox360' | 'dual_shock4'
})

onMounted(() => {
  store.fetchConfig()
})

function openCreateModal() {
  profileForm.value = {
    name: '',
    game_process: '',
    controller_type: 'xbox360'
  }
  showCreateModal.value = true
}

async function handleCreateProfile() {
  if (!profileForm.value.name.trim() || !profileForm.value.game_process.trim()) {
    alert('请填写完整的信息')
    return
  }

  const newProfile: GameProfile = {
    id: 'p_' + Date.now().toString(36) + Math.random().toString(36).substring(2, 7),
    name: profileForm.value.name.trim(),
    game_process: profileForm.value.game_process.trim(),
    controller_type: profileForm.value.controller_type,
    macros: [],
    scripts: []
  }

  store.config.profiles.push(newProfile)
  
  // 如果当前没有激活的 Profile，则自动激活它
  if (!store.config.active_profile) {
    store.config.active_profile = newProfile.id
  }

  await store.saveConfig()
  showCreateModal.value = false
}

async function activateProfile(id: string) {
  store.config.active_profile = id
  await store.saveConfig()
}

async function deactivateProfile() {
  store.config.active_profile = null
  await store.saveConfig()
}

async function handleDeleteProfile(id: string) {
  if (!confirm('确定要删除这个 Profile 吗？')) {
    return
  }

  store.config.profiles = store.config.profiles.filter(p => p.id !== id)
  if (store.config.active_profile === id) {
    store.config.active_profile = null
  }

  await store.saveConfig()
}

function handleExportProfile(profile: GameProfile) {
  const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(profile, null, 2))
  const downloadAnchor = document.createElement('a')
  downloadAnchor.setAttribute("href", dataStr)
  downloadAnchor.setAttribute("download", `${profile.name.replace(/\s+/g, '_')}_profile.json`)
  document.body.appendChild(downloadAnchor)
  downloadAnchor.click()
  downloadAnchor.remove()
}

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
      if (!parsed.name || !parsed.game_process || !parsed.controller_type) {
        alert('导入失败：JSON 格式不正确，缺少必需的字段')
        return
      }

      const newProfile: GameProfile = {
        id: 'p_' + Date.now().toString(36) + Math.random().toString(36).substring(2, 7),
        name: parsed.name,
        game_process: parsed.game_process,
        controller_type: parsed.controller_type === 'dual_shock4' ? 'dual_shock4' : 'xbox360',
        macros: Array.isArray(parsed.macros) ? parsed.macros : [],
        scripts: Array.isArray(parsed.scripts) ? parsed.scripts : []
      }

      store.config.profiles.push(newProfile)
      if (!store.config.active_profile) {
        store.config.active_profile = newProfile.id
      }

      await store.saveConfig()
      alert(`成功导入 Profile: ${newProfile.name}`)
    } catch (err) {
      alert('解析文件失败，请确保导入的是有效的 Profile JSON 文件。')
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
            :class="{ active: store.config.active_profile === profile.id }"
          >
            <div class="profile-icon">
              <Gamepad :size="18" />
            </div>
            
            <div class="profile-main">
              <div class="profile-title">
                <h4>{{ profile.name }}</h4>
                <span class="controller-badge" :class="profile.controller_type">
                  {{ profile.controller_type === 'xbox360' ? 'Xbox 360' : 'DS4' }}
                </span>
                <span v-if="store.config.active_profile === profile.id" class="active-badge">
                  <Check :size="11" /> 已激活
                </span>
              </div>
              <p class="profile-process">{{ profile.game_process }}</p>
            </div>

            <div class="profile-actions">
              <button 
                v-if="store.config.active_profile !== profile.id" 
                class="btn-activate" 
                @click="activateProfile(profile.id)"
              >
                激活
              </button>
              <button 
                v-else 
                class="btn-deactivate" 
                @click="deactivateProfile"
              >
                取消激活
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

    <!-- 创建 Profile 弹窗 (Modal) -->
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
            <label>游戏进程名称 (Game Process)</label>
            <input 
              type="text" 
              v-model="profileForm.game_process" 
              placeholder="例如: ForzaHorizon5.exe" 
              class="form-input" 
              required 
            />
          </div>
          
          <div class="form-group">
            <label>模拟手柄类型</label>
            <select v-model="profileForm.controller_type" class="form-select">
              <option value="xbox360">Xbox 360 (推荐，XInput 兼容性广)</option>
              <option value="dual_shock4">DualShock 4 (PlayStation 4 模式)</option>
            </select>
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="btn-cancel" @click="showCreateModal = false">取消</button>
          <button class="btn-submit" @click="handleCreateProfile">创建</button>
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
}

.profile-card.active .profile-icon {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
}

.profile-main {
  flex-grow: 1;
}

.profile-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: 2px;
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

.controller-badge.dual_shock4 {
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
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

.profile-process {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
  margin: 0;
}

.profile-actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
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

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
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
</style>
