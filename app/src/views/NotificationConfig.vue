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
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardFooter } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, DialogClose } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import PageShell from '@/components/layout/PageShell.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import EmptyState from '@/components/layout/EmptyState.vue'

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
  <PageShell>
    <PageHeader
      title="聚合通知配置"
      description="配置飞书群 Webhook、Server 酱或 Telegram Bot，在任务完成或中断时自动分发通知。"
    >
      <template #actions>
      <Button @click="openCreateDialog">
        <Plus :size="16" class="mr-1" />
        <span>添加通道</span>
      </Button>
      </template>
    </PageHeader>

    <!-- 通道卡片列表 -->
    <div class="flex-1 grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-6 content-start">
      <EmptyState
        v-if="!configStore.config.notification_channels || configStore.config.notification_channels.length === 0"
        title="尚未配置通知通道"
        description="添加通道后，可在定时任务执行完成或中断时发送聚合通知。"
        class="col-span-full min-h-[280px]"
      >
        <template #icon>
          <BellRing :size="42" />
        </template>
        <template #actions>
        <Button variant="outline" @click="openCreateDialog">
          添加首个通道
        </Button>
        </template>
      </EmptyState>

      <Card
        v-else
        v-for="channel in configStore.config.notification_channels"
        :key="channel.id"
        class="transition-all duration-150 hover:-translate-y-0.5 hover:shadow-md hover:border-primary/50"
      >
        <CardHeader class="flex flex-row justify-between items-start pt-1.5 px-4 pb-2">
          <div class="flex flex-col gap-2">
            <CardTitle class="text-sm font-semibold text-foreground">{{ channel.name }}</CardTitle>
            <Badge :class="channel.config.type === 'feishu' ? 'bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/10' : channel.config.type === 'serverchan' ? 'bg-amber-500/10 text-amber-500 hover:bg-amber-500/10' : channel.config.type === 'serverchan3' ? 'bg-red-500/10 text-red-500 hover:bg-red-500/10' : 'bg-blue-500/10 text-blue-500 hover:bg-blue-500/10'" class="self-start text-[10px] font-semibold px-2 py-0.5 uppercase">
              {{ getTypeName(channel.config.type) }}
            </Badge>
          </div>
          <div class="flex gap-0.5">
            <Button variant="ghost" size="icon" class="w-6 h-6" @click="openEditDialog(channel)" title="编辑">
              <Edit2 :size="14" />
            </Button>
            <Button variant="ghost" size="icon" class="w-6 h-6 text-destructive hover:text-destructive hover:bg-destructive/10" @click="handleDelete(channel.id)" title="删除">
              <Trash2 :size="14" />
            </Button>
          </div>
        </CardHeader>

        <CardContent class="px-4 pb-3">
          <div class="flex flex-col gap-1.5 text-xs">
            <div v-if="channel.config.type === 'feishu'" class="flex flex-col gap-0.5">
              <div>
                <span class="text-muted-foreground text-[11px] block">Webhook URL:</span>
                <span class="text-muted-foreground block break-all font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">{{ maskFeishuUrl(channel.config.webhook_url) }}</span>
              </div>
              <div v-if="channel.config.secret">
                <span class="text-muted-foreground text-[11px] block">密钥保护:</span>
                <span class="text-muted-foreground block font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">********</span>
              </div>
            </div>
            <div v-else-if="channel.config.type === 'serverchan'" class="flex flex-col gap-0.5">
              <div>
                <span class="text-muted-foreground text-[11px] block">Send Key:</span>
                <span class="text-muted-foreground block break-all font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">{{ maskSendKey(channel.config.send_key) }}</span>
              </div>
            </div>
            <div v-else-if="channel.config.type === 'serverchan3'" class="flex flex-col gap-0.5">
              <div>
                <span class="text-muted-foreground text-[11px] block">UID:</span>
                <span class="text-muted-foreground block break-all font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">{{ maskUid(channel.config.uid) }}</span>
              </div>
              <div>
                <span class="text-muted-foreground text-[11px] block">Send Key:</span>
                <span class="text-muted-foreground block break-all font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">{{ maskSendKey(channel.config.send_key) }}</span>
              </div>
            </div>
            <div v-else-if="channel.config.type === 'telegram'" class="flex flex-col gap-0.5">
              <div>
                <span class="text-muted-foreground text-[11px] block">Bot Token:</span>
                <span class="text-muted-foreground block break-all font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">{{ maskBotToken(channel.config.bot_token) }}</span>
              </div>
              <div>
                <span class="text-muted-foreground text-[11px] block">Chat ID:</span>
                <span class="text-muted-foreground block break-all font-mono bg-accent/50 px-2 py-1 rounded border border-border/30 text-[11px]">{{ maskChatId(channel.config.chat_id) }}</span>
              </div>
            </div>
          </div>
        </CardContent>

        <CardFooter class="p-4 pt-0">
          <Button
            variant="outline"
            size="sm"
            class="w-full text-xs"
            :class="testStatus[channel.id] === 'success' ? 'border-emerald-500 text-emerald-500 bg-emerald-500/10' : testStatus[channel.id] === 'error' ? 'border-destructive text-destructive bg-destructive/10' : ''"
            :disabled="testStatus[channel.id] === 'testing'"
            @click="handleTest(channel)"
          >
            <span v-if="testStatus[channel.id] === 'testing'" class="flex items-center gap-1.5">
              <span class="w-3 h-3 border-2 border-foreground/50 border-t-transparent rounded-full animate-spin"></span>
              <span>正在测试...</span>
            </span>
            <span v-else-if="testStatus[channel.id] === 'success'" class="flex items-center gap-1.5">
              <CheckCircle2 :size="14" />
              <span>测试成功</span>
            </span>
            <span v-else-if="testStatus[channel.id] === 'error'" class="flex items-center gap-1.5">
              <AlertCircle :size="14" />
              <span>发送失败</span>
            </span>
            <span v-else class="flex items-center gap-1.5">
              <Send :size="12" />
              <span>发送测试消息</span>
            </span>
          </Button>
        </CardFooter>
      </Card>
    </div>

    <!-- 弹窗配置表单 -->
    <Dialog :open="showDialog" @update:open="showDialog = $event">
      <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{{ dialogMode === 'create' ? '新建通知通道' : '编辑通知通道' }}</DialogTitle>
          <DialogDescription>配置通知平台连接信息，保存后即可在定时任务中使用。</DialogDescription>
        </DialogHeader>

        <div class="flex flex-col gap-4 py-2">
          <!-- 名字 -->
          <div class="flex flex-col gap-1.5">
            <Label class="text-xs text-muted-foreground">通道名称</Label>
            <Input
              v-model="formName"
              type="text"
              placeholder="例如：我的飞书挂机群通知"
            />
          </div>

          <!-- 通道类型 -->
          <div class="flex flex-col gap-1.5">
            <Label class="text-xs text-muted-foreground">平台类型</Label>
            <Select v-model="formType" :disabled="dialogMode === 'edit'">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="feishu">飞书群机器人 (Feishu Webhook)</SelectItem>
                <SelectItem value="serverchan">Server酱 (Turbo)</SelectItem>
                <SelectItem value="serverchan3">Server酱³ (V3)</SelectItem>
                <SelectItem value="telegram">Telegram Bot</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- 飞书专属配置 -->
          <div v-if="formType === 'feishu'" class="flex flex-col gap-3 bg-accent/10 border border-border rounded-lg p-4">
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">Webhook URL</Label>
              <Input
                v-model="formFeishuUrl"
                type="text"
                placeholder="https://open.feishu.cn/open-apis/bot/v2/hook/..."
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">
                密钥 Secret
                <span class="text-muted-foreground/60 text-[10px]"> (选填)</span>
              </Label>
              <Input
                v-model="formFeishuSecret"
                type="password"
                placeholder="安全设置中勾选签名校验生成的密文"
              />
            </div>
          </div>

          <!-- Server酱配置 -->
          <div v-if="formType === 'serverchan'" class="flex flex-col gap-3 bg-accent/10 border border-border rounded-lg p-4">
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">SendKey</Label>
              <Input
                v-model="formServerChanKey"
                type="text"
                placeholder="SCT..."
              />
            </div>
          </div>

          <!-- Server酱³ 配置 -->
          <div v-if="formType === 'serverchan3'" class="flex flex-col gap-3 bg-accent/10 border border-border rounded-lg p-4">
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">UID</Label>
              <Input
                v-model="formServerChan3Uid"
                type="text"
                placeholder="从 SendKey 页面获得的 UID，例如 16230"
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">SendKey</Label>
              <Input
                v-model="formServerChanKey"
                type="text"
                placeholder="sctp..."
              />
            </div>
          </div>

          <!-- Telegram 配置 -->
          <div v-if="formType === 'telegram'" class="flex flex-col gap-3 bg-accent/10 border border-border rounded-lg p-4">
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">Bot Token</Label>
              <Input
                v-model="formTelegramToken"
                type="password"
                placeholder="1234567890:ABCdefGhI..."
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <Label class="text-xs text-muted-foreground">Chat ID</Label>
              <Input
                v-model="formTelegramChatId"
                type="text"
                placeholder="例如：987654321 或 @my_channel_id"
              />
            </div>
          </div>
        </div>

        <DialogFooter>
          <DialogClose>
            <Button variant="outline">取消</Button>
          </DialogClose>
          <Button @click="handleSave">保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </PageShell>
</template>
