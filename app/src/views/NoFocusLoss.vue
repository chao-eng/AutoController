<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUIStore } from '../stores/ui'
import { 
  AlertTriangle, 
  RefreshCw, 
  Search, 
  Zap, 
  ZapOff, 
  ShieldAlert, 
  ShieldCheck,
  HelpCircle, 
  Info,
  CheckCircle,
  XCircle
} from '@lucide/vue'
import { Button } from '@/components/ui/button'

interface ProcessInfo {
  pid: number
  name: string
  window_title: string
  is_64bit: boolean
}

const injectableProcesses = ref<ProcessInfo[]>([])
const injectedProcesses = ref<ProcessInfo[]>([])
const searchQuery = ref('')
const loading = ref(false)
const errorMessage = ref<string | null>(null)
const successMessage = ref<string | null>(null)
const isAdmin = ref(false)

const showGuide = ref(false)
const showFeatureGuide = ref(false)

const isAdminTipCollapsed = ref(true)
const isWarningCollapsed = ref(true)

const filteredProcesses = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return injectableProcesses.value
  return injectableProcesses.value.filter(
    (p) =>
      p.pid.toString().includes(query) ||
      p.name.toLowerCase().includes(query) ||
      p.window_title.toLowerCase().includes(query)
  )
})

async function fetchProcesses() {
  loading.value = true
  errorMessage.value = null
  try {
    const injected = await invoke<ProcessInfo[]>('get_injected_processes')
    injectedProcesses.value = injected
    const list = await invoke<ProcessInfo[]>('get_injectable_processes')
    injectableProcesses.value = list
  } catch (err: any) {
    console.error('获取窗口进程列表失败:', err)
    errorMessage.value = `获取窗口进程列表失败: ${err.toString()}`
  } finally {
    loading.value = false
  }
}

async function injectHook(proc: ProcessInfo) {
  errorMessage.value = null
  successMessage.value = null
  try {
    await invoke('inject_focus_hook', { pid: proc.pid, is64bit: proc.is_64bit })
    injectableProcesses.value = injectableProcesses.value.filter((p) => p.pid !== proc.pid)
    injectedProcesses.value.push(proc)
    successMessage.value = `成功将防失去焦点 Hook 注入至进程「${proc.name}」(PID: ${proc.pid})！`
    setTimeout(() => {
      if (successMessage.value?.includes(proc.pid.toString())) {
        successMessage.value = null
      }
    }, 5000)
  } catch (err: any) {
    console.error('注入 Hook 失败:', err)
    errorMessage.value = err.toString()
    if (err.toString().includes('injector.exe') || err.toString().includes('拦截') || err.toString().includes('隔离')) {
      showGuide.value = true
    }
  }
}

async function unloadHook(proc: ProcessInfo) {
  errorMessage.value = null
  successMessage.value = null
  try {
    await invoke('unload_focus_hook', { pid: proc.pid, is64bit: proc.is_64bit })
    injectedProcesses.value = injectedProcesses.value.filter((p) => p.pid !== proc.pid)
    injectableProcesses.value.push(proc)
    injectableProcesses.value.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
    successMessage.value = `成功从进程「${proc.name}」(PID: ${proc.pid}) 中安全卸载 Hook！`
    setTimeout(() => {
      successMessage.value = null
    }, 5000)
  } catch (err: any) {
    console.error('卸载 Hook 失败:', err)
    errorMessage.value = err.toString()
  }
}

const uiStore = useUIStore()

async function handleAutoExclude() {
  try {
    await invoke('add_defender_exclusion')
    uiStore.showToast('🚀 成功将软件运行文件夹添加至 Windows Defender 排除目录！', 'success')
  } catch (err: any) {
    uiStore.showAlert('自动添加失败', `自动排除失败：${err.toString()}\n\n您也可以手动按照下方步骤进行配置。`)
  }
}

async function checkAdminStatus() {
  try {
    isAdmin.value = await invoke<boolean>('check_is_admin')
  } catch (err) {
    console.error('检查管理员权限失败:', err)
  }
}

onMounted(() => {
  fetchProcesses()
  checkAdminStatus()
})
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto bg-background p-6">
    <div class="mb-4 flex shrink-0 items-center justify-between">
      <h2 class="text-lg font-semibold text-foreground">防止游戏/窗口失去焦点 (No Focus Loss)</h2>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" @click="showFeatureGuide = !showFeatureGuide">
          <Info :size="14" class="mr-1" />
          <span>功能使用说明</span>
        </Button>
        <Button variant="outline" size="sm" @click="showGuide = !showGuide">
          <HelpCircle :size="14" class="mr-1" />
          <span>杀软信任指引</span>
        </Button>
        <Button variant="default" size="sm" @click="fetchProcesses" :disabled="loading">
          <RefreshCw :size="14" :class="{ 'animate-spin': loading }" class="mr-1" />
          <span>刷新进程</span>
        </Button>
      </div>
    </div>

    <div v-if="!isAdmin" class="mb-4 flex flex-col gap-2 rounded-lg border border-amber-200 bg-amber-50/50 p-4 text-amber-700">
      <div class="flex items-center gap-2">
        <ShieldAlert :size="16" class="shrink-0" />
        <span class="text-sm font-medium">推荐以管理员权限运行 (Administrator Privileges Recommended)</span>
        <span v-if="isAdminTipCollapsed" class="text-xs text-amber-500">：未以管理员权限启动，注入高权限游戏可能会失败。</span>
        <button class="ml-auto text-xs font-medium text-amber-600 hover:text-amber-800" @click="isAdminTipCollapsed = !isAdminTipCollapsed">
          {{ isAdminTipCollapsed ? '展开详情' : '收起' }}
        </button>
      </div>
      <Transition name="slide">
        <div v-if="!isAdminTipCollapsed" class="text-xs leading-relaxed text-amber-600">
          当前软件<strong>未以管理员身份运行</strong>。由于"防止窗口失焦"功能需要对目标游戏/程序进行跨进程注入，若目标游戏或软件是以管理员权限启动的（例如大部分大型3D游戏或Steam/Wegame平台下的游戏），普通权限的 AutoController 将会因系统权限不足（注入错误代码 102 或卸载错误代码 123）导致操作失败。<strong>强烈建议您右键本程序，选择「以管理员身份运行」重新启动。</strong>
        </div>
      </Transition>
    </div>
    <div v-else class="mb-4 flex flex-col gap-2 rounded-lg border border-emerald-200 bg-emerald-50/50 p-4 text-emerald-700">
      <div class="flex items-center gap-2">
        <ShieldCheck :size="16" class="shrink-0" />
        <span class="text-sm font-medium">已以管理员权限运行 (Running with Administrator Privileges)</span>
        <span v-if="isAdminTipCollapsed" class="text-xs text-emerald-500">：已具备完整的系统高权限，可顺利附加注入。</span>
        <button class="ml-auto text-xs font-medium text-emerald-600 hover:text-emerald-800" @click="isAdminTipCollapsed = !isAdminTipCollapsed">
          {{ isAdminTipCollapsed ? '展开详情' : '收起' }}
        </button>
      </div>
      <Transition name="slide">
        <div v-if="!isAdminTipCollapsed" class="text-xs leading-relaxed text-emerald-600">
          当前软件<strong>已成功以管理员身份运行</strong>。程序已具备完整的系统权限，可以完美支持对高权限游戏及各类窗口程序附加防止失焦 Hook 拦截。
        </div>
      </Transition>
    </div>

    <div class="mb-4 flex flex-col gap-2 rounded-lg border border-red-200 bg-red-50/50 p-4 text-red-700">
      <div class="flex items-center gap-2">
        <AlertTriangle :size="16" class="shrink-0" />
        <span class="text-sm font-medium">高危安全警告与免责声明 (Ban Risk Warning)</span>
        <span v-if="isWarningCollapsed" class="text-xs text-red-500">：跨进程注入在多人网游中有封号风险，严禁在网游中使用！</span>
        <button class="ml-auto text-xs font-medium text-red-600 hover:text-red-800" @click="isWarningCollapsed = !isWarningCollapsed">
          {{ isWarningCollapsed ? '展开详情' : '收起' }}
        </button>
      </div>
      <Transition name="slide">
        <div v-if="!isWarningCollapsed" class="text-xs leading-relaxed text-red-600">
          <p>1. <strong>封号风险警告</strong>：本功能基于跨进程注入技术（DLL Injection）拦截窗口失活消息。这会被反作弊系统（如 EAC、BattlEye、Vanguard 等）视为外挂注入，<strong>在多人网络游戏或带有反作弊保护的游戏中开启此功能有极高封号风险！</strong></p>
          <p>2. <strong>网络游戏禁用</strong>：<strong>严禁在网络联机游戏中使用此功能</strong>。仅推荐在单机游戏（例如单机挂机、防止切屏暂停/静音、双显屏辅助等）中使用。</p>
          <p>3. <strong>免责说明</strong>：本工具为开源辅助软件，因违反规则或在网游中误用导致的任何损失（包括但不限于账号被封禁、处罚）均由使用者本人承担。</p>
        </div>
      </Transition>
    </div>

    <div v-if="errorMessage" class="mb-4 flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-xs text-red-700">
      <XCircle :size="14" class="shrink-0" />
      <span class="flex-1">{{ errorMessage }}</span>
      <button class="text-red-400 hover:text-red-600" @click="errorMessage = null">×</button>
    </div>
    
    <div v-if="successMessage" class="mb-4 flex items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-xs text-emerald-700">
      <CheckCircle :size="14" class="shrink-0" />
      <span class="flex-1">{{ successMessage }}</span>
      <button class="text-emerald-400 hover:text-emerald-600" @click="successMessage = null">×</button>
    </div>

    <Transition name="slide">
      <div v-if="showFeatureGuide" class="mb-4 rounded-lg border border-border bg-card p-4">
        <div class="mb-3 flex items-center gap-2">
          <Info :size="16" class="text-primary" />
          <h4 class="text-sm font-semibold text-foreground">防止游戏失焦功能使用指南 (No Focus Loss Feature Guide)</h4>
        </div>
        <div class="space-y-2 text-xs leading-relaxed text-muted-foreground">
          <p><strong class="text-foreground">什么是防止游戏失焦？</strong><br />
            当您切换到其他工作窗口或将游戏切换至后台时，许多游戏（尤其是使用 Unity、Unreal Engine 等引擎开发的游戏）会自动触发<strong>暂停、画面静止、声音变静音</strong>，或者大幅度降低后台渲染帧率（FPS）。<br />
            本功能通过在底层将轻量级的拦截机制（<code class="rounded bg-muted px-1 py-0.5 font-mono text-xs">NoFocusLoss.dll</code>/<code class="rounded bg-muted px-1 py-0.5 font-mono text-xs">NoFocusLoss64.dll</code>）安全附加到游戏进程中，动态拦截窗口失活消息。<strong>即使您切屏、查看网页或多屏操作，游戏在后台也能保持与前台完全相同的满帧渲染、声音播放及挂机运行状态。</strong></p>
          
          <h5 class="mt-3 text-xs font-bold text-foreground">使用步骤与指引：</h5>
          <ol class="list-inside list-decimal space-y-1">
            <li><strong>管理员身份运行（强烈推荐）</strong>：由于高级别游戏具备高系统权限，请确保以<strong>管理员身份</strong>运行 AutoController，否则注入器会因权限不足而失败。</li>
            <li><strong>游戏必须窗口化</strong>：目标游戏需要在<strong>窗口化</strong>或<strong>无边框窗口化（Borderless）</strong>模式下运行，在独占全屏下无法发挥作用。<strong>（注意：部分游戏在修改为窗口化后，必须重启游戏才能使该渲染模式生效，建议配置后重启游戏再进行注入）</strong></li>
            <li><strong>一键注入挂机</strong>：在左侧进程列表搜索游戏并点击<strong>「注入 Hook」</strong>，即可激活后台挂机模式！</li>
            <li><strong>安全一键剥离</strong>：挂机结束后，可在右侧列表随时点击<strong>「安全卸载」</strong>。系统会自动干净地释放全部游戏内存占用，无痕恢复游戏默认行为。</li>
          </ol>
        </div>
      </div>
    </Transition>

    <Transition name="slide">
      <div v-if="showGuide" class="mb-4 rounded-lg border border-border bg-card p-4">
        <div class="mb-3 flex items-center gap-2">
          <ShieldAlert :size="16" class="text-destructive" />
          <h4 class="text-sm font-semibold text-foreground">Windows Defender 杀软拦截修复指引</h4>
        </div>
        <div class="space-y-2 text-xs leading-relaxed text-muted-foreground">
          <p>由于本功能采用<strong>"物理隔离"</strong>技术（由独立子进程 <code class="rounded bg-muted px-1 py-0.5 font-mono text-xs">injector.exe</code> 动态解密并执行注入，彻底避免主程序崩溃或被报毒），Windows Defender 或杀毒软件可能会对 <code class="rounded bg-muted px-1 py-0.5 font-mono text-xs">injector.exe</code> 进行警报或拦截。请按照以下步骤添加排除项：</p>
          
          <div v-if="isAdmin" class="my-2 flex items-center justify-between gap-4 rounded-lg border border-dashed border-primary/25 bg-primary/5 p-3">
            <div class="text-xs leading-relaxed text-muted-foreground">
              <strong class="text-foreground">🛡️ 已检测到管理员权限：</strong><br />
              支持一键调用 PowerShell 静默将本软件当前运行目录自动添加至 Defender 信任区。
            </div>
            <Button size="sm" @click="handleAutoExclude">
              ⚡ 一键自动添加信任排除
            </Button>
          </div>

          <ol class="list-inside list-decimal space-y-1">
            <li>打开 Windows 的 <strong>「安全中心」</strong> ➔ <strong>「病毒和威胁防护」</strong>。</li>
            <li>点击 <strong>「"病毒和威胁防护"设置」</strong> 下方的 <strong>「管理设置」</strong>。</li>
            <li>向下滑动到 <strong>「排除项」</strong>，点击 <strong>「添加或删除排除项」</strong>。</li>
            <li>点击 <strong>「添加排除项」</strong> ➔ 选择 <strong>「文件」</strong>。</li>
            <li>定位到 AutoController 的安装目录（或开发环境的 target 目录），选中 <code class="rounded bg-muted px-1 py-0.5 font-mono text-xs">injector.exe</code> 并点击添加即可。</li>
          </ol>
          <div class="mt-2 flex items-center gap-1 text-muted-foreground">
            <Info :size="12" />
            <span>提示：我们的注入器代码完全开源，绝无后门及恶意行为，您可以放心添加信任运行。</span>
          </div>
        </div>
      </div>
    </Transition>

    <div class="grid flex-1 grid-cols-2 gap-4 overflow-hidden">
      <div class="flex flex-col overflow-hidden rounded-lg border border-border bg-card">
        <div class="flex items-center justify-between border-b border-border px-4 py-3">
          <h3 class="flex items-center gap-2 text-sm font-semibold text-foreground">
            <span>活动窗口进程 (可注入)</span>
            <span class="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary/10 px-1.5 text-[11px] font-medium text-primary">{{ filteredProcesses.length }}</span>
          </h3>
          <div class="flex items-center gap-2">
            <div class="flex items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1">
              <Search :size="12" class="text-muted-foreground" />
              <input v-model="searchQuery" type="text" placeholder="搜索 PID、进程或窗口标题..." class="w-36 bg-transparent text-xs text-foreground outline-none placeholder:text-muted-foreground" />
            </div>
            <Button variant="ghost" size="icon-sm" @click="fetchProcesses" :disabled="loading" title="刷新列表">
              <RefreshCw :size="14" :class="{ 'animate-spin': loading }" />
            </Button>
          </div>
        </div>
        
        <div class="flex-1 overflow-y-auto">
          <div v-if="filteredProcesses.length === 0" class="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
            <Info :size="24" />
            <p class="text-xs">{{ searchQuery ? '未找到符合条件的活动窗口' : '暂无符合附加条件的活动窗口，请点击刷新' }}</p>
          </div>
          
          <div v-else class="divide-y divide-border">
            <div v-for="proc in filteredProcesses" :key="proc.pid" class="flex items-center gap-3 px-4 py-3 hover:bg-accent/50">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-foreground">{{ proc.name }}</span>
                  <span class="inline-flex items-center rounded-full bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">PID: {{ proc.pid }}</span>
                  <span class="inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium" :class="proc.is_64bit ? 'bg-blue-100 text-blue-700' : 'bg-amber-100 text-amber-700'">
                    {{ proc.is_64bit ? '64-bit' : '32-bit' }}
                  </span>
                </div>
                <div class="mt-0.5 truncate text-xs text-muted-foreground" :title="proc.window_title">
                  窗口: {{ proc.window_title }}
                </div>
              </div>
              <Button variant="default" size="sm" class="shrink-0 gap-1" @click="injectHook(proc)">
                <Zap :size="12" />
                <span>注入 Hook</span>
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col overflow-hidden rounded-lg border border-border bg-card">
        <div class="flex items-center justify-between border-b border-border px-4 py-3">
          <h3 class="flex items-center gap-2 text-sm font-semibold text-foreground">
            <span>已附加 Hook 进程 (运行中)</span>
            <span class="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-emerald-100 px-1.5 text-[11px] font-medium text-emerald-700">{{ injectedProcesses.length }}</span>
          </h3>
        </div>

        <div class="flex-1 overflow-y-auto">
          <div v-if="injectedProcesses.length === 0" class="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
            <ZapOff :size="24" />
            <p class="text-xs">当前无附加 Hook 状态的进程</p>
            <span class="text-[11px] text-muted-foreground/60">在左侧列表中选择进程并点击"注入 Hook"</span>
          </div>

          <div v-else class="divide-y divide-border">
            <div v-for="proc in injectedProcesses" :key="proc.pid" class="flex items-center gap-3 bg-emerald-50/30 px-4 py-3 hover:bg-emerald-50/50">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-foreground">{{ proc.name }}</span>
                  <span class="inline-flex items-center rounded-full bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">PID: {{ proc.pid }}</span>
                  <span class="inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium" :class="proc.is_64bit ? 'bg-blue-100 text-blue-700' : 'bg-amber-100 text-amber-700'">
                    {{ proc.is_64bit ? '64-bit' : '32-bit' }}
                  </span>
                </div>
                <div class="mt-0.5 truncate text-xs text-muted-foreground" :title="proc.window_title">
                  窗口: {{ proc.window_title }}
                </div>
              </div>
              <Button variant="outline" size="sm" class="shrink-0 gap-1 border-amber-200 text-amber-700 hover:bg-amber-50" @click="unloadHook(proc)">
                <ZapOff :size="12" />
                <span>安全卸载</span>
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>