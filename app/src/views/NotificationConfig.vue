<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigStore } from '../stores/config'
import type { NotificationChannel } from '../types/config'
import { invoke } from '@tauri-apps/api/core'
import {
  Plus,
  Send,
  Trash2,
  Edit2,
  BellRing,
  AlertCircle,
  CheckCircle2,
} from '@lucide/vue'

import { useUIStore } from '../stores/ui'

const configStore = useConfigStore()
const uiStore = useUIStore()

// 敏感配置脱敏函数
function maskFeishuUrl(url: string | undefined): string {
  if (!url) return ''
  const hookStr = '/hook/'
  const idx = url.indexOf(hookStr)
  if (idx !== -1) {
    return url.substring(0, idx + hookStr.length) + '********'
  }
  return '********'
}

function maskSendKey(key: string | undefined): string {
  if (!key) return ''
  if (key.startsWith('SCT')) return 'SCT********'
  if (key.startsWith('sctp')) return 'sctp********'
  return '********'
}

function maskUid(uid: string | undefined): string {
  if (!uid) return ''
  if (uid.length > 2) {
    return uid.substring(0, 2) + '***'
  }
  return '***'
}

function maskBotToken(token: string | undefined): string {
  if (!token) return ''
  const idx = token.indexOf(':')
  if (idx !== -1) {
    return token.substring(0, idx + 1) + '********'
  }
  return '********'
}

function maskChatId(id: string | undefined): string {
  if (!id) return ''
  if (id.startsWith('-')) {
    return '-********'
  }
  if (id.length > 3) {
    return id.substring(0, 3) + '********'
  }
  return '********'
}

const showDialog = ref(false)
const dialogMode = ref<'create' | 'edit'>('create')
const editingId = ref<string | null>(null)

// 表单状态
const formName = ref('')
const formType = ref<'feishu' | 'serverchan' | 'serverchan3' | 'telegram'>('feishu')
const formFeishuUrl = ref('')
const formFeishuSecret = ref('')
const formServerChanKey = ref('')
const formServerChan3Uid = ref('')
const formTelegramToken = ref('')
const formTelegramChatId = ref('')

// 测试发送状态
const testStatus = ref<Record<string, 'idle' | 'testing' | 'success' | 'error'>>({})
const testMessage = ref('')

onMounted(async () => {
  await configStore.fetchConfig()
})

function openCreateDialog() {
  dialogMode.value = 'create'
  editingId.value = null
  formName.value = ''
  formType.value = 'feishu'
  formFeishuUrl.value = ''
  formFeishuSecret.value = ''
  formServerChanKey.value = ''
  formServerChan3Uid.value = ''
  formTelegramToken.value = ''
  formTelegramChatId.value = ''
  showDialog.value = true
}

function openEditDialog(channel: NotificationChannel) {
  dialogMode.value = 'edit'
  editingId.value = channel.id
  formName.value = channel.name
  formType.value = channel.config.type
  
  formFeishuUrl.value = channel.config.webhook_url || ''
  formFeishuSecret.value = channel.config.secret || ''
  formServerChanKey.value = channel.config.send_key || ''
  formServerChan3Uid.value = channel.config.uid || ''
  formTelegramToken.value = channel.config.bot_token || ''
  formTelegramChatId.value = channel.config.chat_id || ''
  
  showDialog.value = true
}

async function handleSave() {
  if (!formName.value.trim()) {
    uiStore.showToast('请输入通道名称', 'warning')
    return
  }

  // 基础校验
  if (formType.value === 'feishu' && !formFeishuUrl.value.trim()) {
    uiStore.showToast('请输入飞书 Webhook URL', 'warning')
    return
  }
  if (formType.value === 'serverchan' && !formServerChanKey.value.trim()) {
    uiStore.showToast('请输入 Server酱 SendKey', 'warning')
    return
  }
  if (formType.value === 'serverchan3') {
    if (!formServerChan3Uid.value.trim()) {
      uiStore.showToast('请输入 Server酱³ UID', 'warning')
      return
    }
    if (!formServerChanKey.value.trim()) {
      uiStore.showToast('请输入 Server酱³ SendKey', 'warning')
      return
    }
  }
  if (formType.value === 'telegram' && (!formTelegramToken.value.trim() || !formTelegramChatId.value.trim())) {
    uiStore.showToast('请输入 Telegram Bot Token 和 Chat ID', 'warning')
    return
  }

  const channels = [...(configStore.config.notification_channels || [])]

  const configObj: any = { type: formType.value }
  if (formType.value === 'feishu') {
    configObj.webhook_url = formFeishuUrl.value.trim()
    if (formFeishuSecret.value.trim()) {
      configObj.secret = formFeishuSecret.value.trim()
    }
  } else if (formType.value === 'serverchan') {
    configObj.send_key = formServerChanKey.value.trim()
  } else if (formType.value === 'serverchan3') {
    configObj.uid = formServerChan3Uid.value.trim()
    configObj.send_key = formServerChanKey.value.trim()
  } else if (formType.value === 'telegram') {
    configObj.bot_token = formTelegramToken.value.trim()
    configObj.chat_id = formTelegramChatId.value.trim()
  }

  if (dialogMode.value === 'create') {
    const newChannel: NotificationChannel = {
      id: crypto.randomUUID(),
      name: formName.value.trim(),
      config: configObj
    }
    channels.push(newChannel)
  } else {
    const idx = channels.findIndex(c => c.id === editingId.value)
    if (idx !== -1) {
      channels[idx] = {
        id: editingId.value!,
        name: formName.value.trim(),
        config: configObj
      }
    }
  }

  configStore.config.notification_channels = channels
  await configStore.saveConfig()
  showDialog.value = false
}

async function handleDelete(id: string) {
  const confirmed = await uiStore.showConfirm('确认删除', '确认要删除该通知通道吗？')
  if (!confirmed) return
  const channels = [...(configStore.config.notification_channels || [])].filter(c => c.id !== id)
  configStore.config.notification_channels = channels
  await configStore.saveConfig()
}

async function handleTest(channel: NotificationChannel) {
  testStatus.value[channel.id] = 'testing'
  testMessage.value = ''
  
  try {
    const payload = {
      title: 'AutoController 测试通知',
      content: `这这是一条来自 AutoController 的测试通知。\n配置通道：${channel.name}\n测试时间：${new Date().toLocaleString()}`
    }
    
    await invoke('send_aggregated_notification', {
      channels: [channel.config],
      payload
    })
    
    testStatus.value[channel.id] = 'success'
    setTimeout(() => {
      testStatus.value[channel.id] = 'idle'
    }, 3000)
  } catch (e) {
    testStatus.value[channel.id] = 'error'
    testMessage.value = String(e)
    uiStore.showAlert('通知测试失败', String(e))
    setTimeout(() => {
      testStatus.value[channel.id] = 'idle'
    }, 5000)
  }
}

function getTypeName(type: string): string {
  switch (type) {
    case 'feishu': return '飞书群机器人'
    case 'serverchan': return 'Server酱'
    case 'serverchan3': return 'Server酱³'
    case 'telegram': return 'Telegram Bot'
    default: return type
  }
}
</script>

<template>
  <div class="notify-config">
    <!-- 头部说明 -->
    <div class="page-header">
      <div class="header-title">
        <h2>聚合通知配置</h2>
        <p class="subtitle">配置飞书群Webhook、Server酱或Telegram Bot，在定时任务序列执行中断或完成时自动向您分发通知。</p>
      </div>
      <button class="cta-btn" @click="openCreateDialog">
        <Plus :size="16" />
        <span>添加通道</span>
      </button>
    </div>

    <!-- 通道卡片列表 -->
    <div class="channels-grid">
      <div 
        v-if="!configStore.config.notification_channels || configStore.config.notification_channels.length === 0" 
        class="empty-state"
      >
        <BellRing :size="48" class="empty-icon" />
        <h3>尚未配置通知通道</h3>
        <p>点击右上角“添加通道”按钮，立即绑定通知分发服务。</p>
        <button class="cta-btn secondary" @click="openCreateDialog">
          添加首个通道
        </button>
      </div>

      <div 
        v-else 
        v-for="channel in configStore.config.notification_channels" 
        :key="channel.id" 
        class="channel-card"
      >
        <div class="card-header">
          <div class="channel-info">
            <span class="channel-type-tag" :class="channel.config.type">
              {{ getTypeName(channel.config.type) }}
            </span>
            <h3 class="channel-name">{{ channel.name }}</h3>
          </div>
          <div class="card-actions">
            <button class="action-icon-btn" @click="openEditDialog(channel)" title="编辑">
              <Edit2 :size="14" />
            </button>
            <button class="action-icon-btn delete" @click="handleDelete(channel.id)" title="删除">
              <Trash2 :size="14" />
            </button>
          </div>
        </div>

        <div class="card-body">
          <div v-if="channel.config.type === 'feishu'" class="config-summary">
            <div class="info-row">
              <span class="label">Webhook URL:</span>
              <span class="value">{{ maskFeishuUrl(channel.config.webhook_url) }}</span>
            </div>
            <div v-if="channel.config.secret" class="info-row">
              <span class="label">密钥保护:</span>
              <span class="value">********</span>
            </div>
          </div>
          <div v-else-if="channel.config.type === 'serverchan'" class="config-summary">
            <div class="info-row">
              <span class="label">Send Key:</span>
              <span class="value">{{ maskSendKey(channel.config.send_key) }}</span>
            </div>
          </div>
          <div v-else-if="channel.config.type === 'serverchan3'" class="config-summary">
            <div class="info-row">
              <span class="label">UID:</span>
              <span class="value">{{ maskUid(channel.config.uid) }}</span>
            </div>
            <div class="info-row">
              <span class="label">Send Key:</span>
              <span class="value">{{ maskSendKey(channel.config.send_key) }}</span>
            </div>
          </div>
          <div v-else-if="channel.config.type === 'telegram'" class="config-summary">
            <div class="info-row">
              <span class="label">Bot Token:</span>
              <span class="value">{{ maskBotToken(channel.config.bot_token) }}</span>
            </div>
            <div class="info-row">
              <span class="label">Chat ID:</span>
              <span class="value">{{ maskChatId(channel.config.chat_id) }}</span>
            </div>
          </div>
        </div>

        <div class="card-footer">
          <button 
            class="test-btn" 
            :class="testStatus[channel.id]"
            :disabled="testStatus[channel.id] === 'testing'"
            @click="handleTest(channel)"
          >
            <span v-if="testStatus[channel.id] === 'testing'" class="loader-container">
              <span class="mini-loader"></span>
              <span>正在测试...</span>
            </span>
            <span v-else-if="testStatus[channel.id] === 'success'" class="success-container">
              <CheckCircle2 :size="14" />
              <span>测试成功</span>
            </span>
            <span v-else-if="testStatus[channel.id] === 'error'" class="error-container">
              <AlertCircle :size="14" />
              <span>发送失败</span>
            </span>
            <span v-else class="normal-container">
              <Send :size="12" />
              <span>发送测试消息</span>
            </span>
          </button>
        </div>
      </div>
    </div>

    <!-- 弹窗配置表单 -->
    <div v-if="showDialog" class="modal-overlay">
      <div class="modal-content">
        <div class="modal-header">
          <h3>{{ dialogMode === 'create' ? '新建通知通道' : '编辑通知通道' }}</h3>
          <button class="close-btn" @click="showDialog = false">&times;</button>
        </div>

        <div class="modal-body">
          <!-- 名字 -->
          <div class="form-item">
            <label>通道名称</label>
            <input 
              v-model="formName" 
              type="text" 
              placeholder="例如：我的飞书挂机群通知" 
              class="form-input"
            />
          </div>

          <!-- 通道类型 -->
          <div class="form-item">
            <label>平台类型</label>
            <select v-model="formType" class="form-select" :disabled="dialogMode === 'edit'">
              <option value="feishu">飞书群机器人 (Feishu Webhook)</option>
              <option value="serverchan">Server酱 (Turbo)</option>
              <option value="serverchan3">Server酱³ (V3)</option>
              <option value="telegram">Telegram Bot</option>
            </select>
          </div>

          <!-- 飞书专属配置 -->
          <div v-if="formType === 'feishu'" class="platform-specific-fields">
            <div class="form-item">
              <label>Webhook URL</label>
              <input 
                v-model="formFeishuUrl" 
                type="text" 
                placeholder="https://open.feishu.cn/open-apis/bot/v2/hook/..." 
                class="form-input"
              />
            </div>
            <div class="form-item">
              <label class="label-optional">密钥 Secret (选填)</label>
              <input 
                v-model="formFeishuSecret" 
                type="password" 
                placeholder="安全设置中勾选签名校验生成的密文" 
                class="form-input"
              />
            </div>
          </div>

          <!-- Server酱配置 -->
          <div v-if="formType === 'serverchan'" class="platform-specific-fields">
            <div class="form-item">
              <label>SendKey</label>
              <input 
                v-model="formServerChanKey" 
                type="text" 
                placeholder="SCT..." 
                class="form-input"
              />
            </div>
          </div>

          <!-- Server酱³ 配置 -->
          <div v-if="formType === 'serverchan3'" class="platform-specific-fields">
            <div class="form-item">
              <label>UID</label>
              <input 
                v-model="formServerChan3Uid" 
                type="text" 
                placeholder="从 SendKey 页面获得的 UID，例如 16230" 
                class="form-input"
              />
            </div>
            <div class="form-item">
              <label>SendKey</label>
              <input 
                v-model="formServerChanKey" 
                type="text" 
                placeholder="sctp..." 
                class="form-input"
              />
            </div>
          </div>

          <!-- Telegram 配置 -->
          <div v-if="formType === 'telegram'" class="platform-specific-fields">
            <div class="form-item">
              <label>Bot Token</label>
              <input 
                v-model="formTelegramToken" 
                type="password" 
                placeholder="1234567890:ABCdefGhI..." 
                class="form-input"
              />
            </div>
            <div class="form-item">
              <label>Chat ID</label>
              <input 
                v-model="formTelegramChatId" 
                type="text" 
                placeholder="例如：987654321 或 @my_channel_id" 
                class="form-input"
              />
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-secondary" @click="showDialog = false">取消</button>
          <button class="btn btn-primary" @click="handleSave">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notify-config {
  padding: var(--space-lg);
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-lg);
  flex-shrink: 0;
}

.header-title h2 {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--space-xs);
}

.subtitle {
  font-size: 13px;
  color: var(--color-text-muted);
}

.cta-btn {
  background: var(--color-cta);
  color: white;
  border-radius: var(--radius-md);
  padding: var(--space-sm) var(--space-lg);
  font-size: 13px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.cta-btn:hover {
  background: #2563EB;
}

.cta-btn.secondary {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  margin: var(--space-md) auto 0 auto;
}

.cta-btn.secondary:hover {
  background: var(--color-border);
}

.channels-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--space-lg);
  align-content: start;
}

.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: var(--space-2xl) var(--space-lg);
  color: var(--color-text-dim);
  background: var(--color-surface);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
}

.empty-icon {
  color: var(--color-text-dim);
  opacity: 0.5;
  margin-bottom: var(--space-sm);
}

.empty-state h3 {
  font-size: 16px;
  color: var(--color-text);
}

.empty-state p {
  font-size: 13px;
  max-width: 400px;
  margin: 0 auto;
}

/* 卡片 */
.channel-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  transition: transform var(--transition-fast), box-shadow var(--transition-fast), border-color var(--transition-fast);
}

.channel-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
  border-color: var(--color-cta);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: var(--space-md);
}

.channel-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.channel-type-tag {
  align-self: start;
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
}

.channel-type-tag.feishu {
  background: rgba(16, 185, 129, 0.1);
  color: #10B981;
}

.channel-type-tag.serverchan {
  background: rgba(245, 158, 11, 0.1);
  color: #F59E0B;
}

.channel-type-tag.serverchan3 {
  background: rgba(239, 68, 68, 0.1);
  color: #EF4444;
}

.channel-type-tag.telegram {
  background: rgba(59, 130, 246, 0.1);
  color: #3B82F6;
}

.channel-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.card-actions {
  display: flex;
  gap: var(--space-xs);
}

.action-icon-btn {
  width: 26px;
  height: 26px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-icon-btn:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.action-icon-btn.delete:hover {
  background: rgba(239, 68, 68, 0.1);
  color: var(--color-error);
}

.card-body {
  flex: 1;
  margin-bottom: var(--space-lg);
}

.config-summary {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  font-size: 12px;
}

.info-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.info-row .label {
  color: var(--color-text-dim);
  font-size: 11px;
}

.info-row .value {
  color: var(--color-text-muted);
  word-break: break-all;
  font-family: Consolas, Monaco, monospace;
  background: var(--color-surface-elevated);
  padding: 4px var(--space-sm);
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.03);
}

.card-footer {
  display: flex;
  justify-content: flex-end;
}

.test-btn {
  font-size: 12px;
  padding: 6px var(--space-md);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.test-btn:hover:not(:disabled) {
  background: var(--color-border);
  color: var(--color-text);
}

.test-btn.testing {
  opacity: 0.8;
  cursor: not-allowed;
}

.test-btn.success {
  background: rgba(16, 185, 129, 0.15);
  border-color: #10B981;
  color: #10B981;
}

.test-btn.error {
  background: rgba(239, 68, 68, 0.15);
  border-color: var(--color-error);
  color: var(--color-error);
}

.loader-container, .success-container, .error-container, .normal-container {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.mini-loader {
  width: 12px;
  height: 12px;
  border: 2px solid var(--color-text-dim);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 弹窗配置表单 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal-content {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  width: 100%;
  max-width: 500px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
  animation: modalSlide 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  display: flex;
  flex-direction: column;
}

@keyframes modalSlide {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.modal-header {
  padding: var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.close-btn {
  font-size: 24px;
  color: var(--color-text-dim);
  cursor: pointer;
}

.modal-body {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.form-item label {
  font-size: 12px;
  color: var(--color-text-dim);
}

.form-item .label-optional::after {
  content: ' (可选)';
  color: var(--color-text-muted);
  font-size: 10px;
}

.form-input, .form-select {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm) var(--space-md);
  font-size: 13px;
  color: var(--color-text);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-input:focus, .form-select:focus {
  border-color: var(--color-cta);
}

.platform-specific-fields {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  background: rgba(255, 255, 255, 0.01);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
}

.modal-footer {
  padding: var(--space-lg);
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-sm);
}

.btn {
  padding: var(--space-sm) var(--space-lg);
  font-size: 13px;
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-border);
}

.btn-primary {
  background: var(--color-cta);
  color: white;
}

.btn-primary:hover {
  background: #2563EB;
}
</style>
