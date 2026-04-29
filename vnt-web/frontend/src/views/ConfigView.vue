<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { deleteConfig, getConfig, saveConfig } from '@/api/vnt'
import SectionCard from '@/components/SectionCard.vue'
import { useAppStore } from '@/stores/app'
import type { VisualConfig } from '@/types/config'
import {
  createDefaultVisualConfig,
  parseTomlConfig,
  splitLines,
  stringifyVisualConfig,
  validateVisualConfig,
  visualConfigFingerprint,
} from '@/utils/config'

type EditorMode = 'visual' | 'file'

const app = useAppStore()

const selectedFile = ref('')
const draftFileName = ref('')
const editorMode = ref<EditorMode>('visual')
const visualConfig = ref<VisualConfig>(createDefaultVisualConfig())
const editorContent = ref(stringifyVisualConfig(visualConfig.value))
const loadingEditor = ref(false)
const parseError = ref('')

const loadedRawContent = ref('')
const loadedVisualFingerprint = ref('')

const normalizedFileName = computed(() => {
  const value = draftFileName.value.trim()
  if (!value) {
    return ''
  }

  return value.endsWith('.toml') ? value : `${value}.toml`
})

const generatedToml = computed(() => stringifyVisualConfig(visualConfig.value))

const visualDirty = computed(
  () =>
    visualConfigFingerprint(visualConfig.value) !== loadedVisualFingerprint.value ||
    normalizedFileName.value !== selectedFile.value,
)

const rawDirty = computed(
  () =>
    editorContent.value !== loadedRawContent.value ||
    normalizedFileName.value !== selectedFile.value,
)

const isDirty = computed(() => (editorMode.value === 'visual' ? visualDirty.value : rawDirty.value))

const serverText = computed({
  get: () => visualConfig.value.server.join('\n'),
  set: (value: string) => {
    visualConfig.value.server = splitLines(value)
  },
})

const outputText = computed({
  get: () => visualConfig.value.output.join('\n'),
  set: (value: string) => {
    visualConfig.value.output = splitLines(value)
  },
})

const portMappingText = computed({
  get: () => visualConfig.value.port_mapping.join('\n'),
  set: (value: string) => {
    visualConfig.value.port_mapping = splitLines(value)
  },
})

const udpStunText = computed({
  get: () => visualConfig.value.udp_stun.join('\n'),
  set: (value: string) => {
    visualConfig.value.udp_stun = splitLines(value)
  },
})

const tcpStunText = computed({
  get: () => visualConfig.value.tcp_stun.join('\n'),
  set: (value: string) => {
    visualConfig.value.tcp_stun = splitLines(value)
  },
})

function resetLoadedState(fileName: string, content: string, config: VisualConfig) {
  selectedFile.value = fileName
  draftFileName.value = fileName
  editorContent.value = content
  visualConfig.value = config
  loadedRawContent.value = content
  loadedVisualFingerprint.value = visualConfigFingerprint(config)
  parseError.value = ''
}

function updateRawFromVisual(showNotice = true) {
  editorContent.value = generatedToml.value
  parseError.value = ''
  if (showNotice) {
    app.setNotice('success', '已根据图形配置生成 TOML 文件内容。')
  }
}

function updateVisualFromRaw(showNotice = true) {
  try {
    const parsed = parseTomlConfig(editorContent.value)
    visualConfig.value = parsed
    parseError.value = ''
    if (showNotice) {
      app.setNotice('success', '已从 TOML 文件内容解析为图形化配置。')
    }
    return true
  } catch (error) {
    parseError.value = (error as Error).message
    app.setNotice('error', `当前 TOML 无法解析为图形配置：${parseError.value}`, 5600)
    return false
  }
}

function setEditorMode(mode: EditorMode) {
  if (mode === editorMode.value) {
    return
  }

  if (mode === 'visual') {
    if (!updateVisualFromRaw(false)) {
      return
    }
  } else {
    updateRawFromVisual(false)
  }

  editorMode.value = mode
}

async function loadConfig(fileName: string, force = false) {
  if (!force && isDirty.value && !window.confirm('当前内容还没有保存，确定切换配置吗？')) {
    return
  }

  loadingEditor.value = true
  try {
    const content = await getConfig(fileName)
    const parsed = parseTomlConfig(content)
    resetLoadedState(fileName, content, parsed)
  } catch (error) {
    app.setNotice('error', (error as Error).message, 4800)
  } finally {
    loadingEditor.value = false
  }
}

function createNewConfig() {
  if (!isDirty.value || window.confirm('当前内容还没有保存，确定创建新配置草稿吗？')) {
    const stamp = new Date().toISOString().replaceAll(':', '-').slice(0, 19)
    const nextConfig = createDefaultVisualConfig()
    selectedFile.value = ''
    draftFileName.value = `config-${stamp}.toml`
    visualConfig.value = nextConfig
    editorContent.value = stringifyVisualConfig(nextConfig)
    loadedRawContent.value = ''
    loadedVisualFingerprint.value = ''
    parseError.value = ''
    editorMode.value = 'visual'
  }
}

function validateBeforeSave() {
  if (editorMode.value === 'visual') {
    const message = validateVisualConfig(visualConfig.value)
    if (message) {
      app.setNotice('error', message, 5200)
      return ''
    }

    updateRawFromVisual(false)
    return generatedToml.value
  }

  try {
    const parsed = parseTomlConfig(editorContent.value)
    visualConfig.value = parsed
    parseError.value = ''
    return editorContent.value
  } catch (error) {
    parseError.value = (error as Error).message
    app.setNotice('error', `TOML 解析失败：${parseError.value}`, 5600)
    return ''
  }
}

async function saveCurrentConfig() {
  const fileName = normalizedFileName.value
  if (!fileName) {
    app.setNotice('error', '请先填写配置文件名。')
    return false
  }

  const content = validateBeforeSave()
  if (!content) {
    return false
  }

  loadingEditor.value = true
  try {
    await saveConfig(fileName, content)
    await app.fetchConfigList()
    selectedFile.value = fileName
    draftFileName.value = fileName
    loadedRawContent.value = content
    loadedVisualFingerprint.value = visualConfigFingerprint(visualConfig.value)
    app.setNotice('success', `已保存 ${fileName}`)
    return true
  } catch (error) {
    app.setNotice('error', (error as Error).message, 5000)
    return false
  } finally {
    loadingEditor.value = false
  }
}

async function removeCurrentConfig() {
  if (!selectedFile.value) {
    app.setNotice('error', '当前还没有已保存的配置。')
    return
  }

  if (!window.confirm(`确认删除 ${selectedFile.value} 吗？`)) {
    return
  }

  loadingEditor.value = true
  try {
    await deleteConfig(selectedFile.value)
    await app.fetchConfigList()
    if (app.configList.value.length > 0) {
      await loadConfig(app.configList.value[0].file_name, true)
    } else {
      createNewConfig()
    }
    app.setNotice('success', '配置已删除。')
  } catch (error) {
    app.setNotice('error', (error as Error).message, 5000)
  } finally {
    loadingEditor.value = false
  }
}

async function saveIfDirty() {
  if (!isDirty.value) {
    return true
  }

  return saveCurrentConfig()
}

async function startCurrent() {
  const ok = await saveIfDirty()
  if (!ok || !normalizedFileName.value) {
    return
  }

  await app.start(normalizedFileName.value)
}

async function restartCurrent() {
  const ok = await saveIfDirty()
  if (!ok || !normalizedFileName.value) {
    return
  }

  await app.restart(normalizedFileName.value)
}

function addInputRule() {
  visualConfig.value.input.push({
    net: '',
    target_ip: '',
  })
}

function removeInputRule(index: number) {
  visualConfig.value.input.splice(index, 1)
}

watch(
  () => app.configList.value,
  async (configs) => {
    if (selectedFile.value || configs.length === 0) {
      return
    }

    const preferred =
      configs.find((item) => item.file_name === app.info.value.current_config_file) ?? configs[0]
    await loadConfig(preferred.file_name, true)
  },
  { immediate: true },
)

onMounted(() => {
  if (app.configList.value.length === 0 && !selectedFile.value) {
    createNewConfig()
  }
})
</script>

<template>
  <div class="config-layout">
    <SectionCard title="配置文件" subtitle="保留配置索引，支持草稿、新建、切换和删除。">
      <template #action>
        <div class="inline-actions">
          <button class="button ghost" @click="app.fetchConfigList()">刷新列表</button>
          <button class="button primary" @click="createNewConfig()">新建配置</button>
        </div>
      </template>

      <div class="config-list">
        <button
          v-for="config in app.configList.value"
          :key="config.file_name"
          class="config-item"
          :class="{ active: config.file_name === selectedFile }"
          @click="loadConfig(config.file_name)"
        >
          <strong>{{ config.config_name }}</strong>
          <span>{{ config.file_name }}</span>
        </button>

        <div v-if="app.configList.value.length === 0" class="empty-state">
          还没有配置文件，点击“新建配置”先生成一个草稿。
        </div>
      </div>
    </SectionCard>

    <div class="editor-stack">
      <SectionCard title="编辑模式" subtitle="图形化和原始 TOML 双模式共存，可以随时互相同步。">
        <template #action>
          <div class="inline-actions">
            <button class="button ghost" @click="setEditorMode('visual')">图形化配置</button>
            <button class="button ghost" @click="setEditorMode('file')">文件配置器</button>
          </div>
        </template>

        <div class="editor-toolbar">
          <label class="field">
            <span>文件名</span>
            <input v-model="draftFileName" type="text" placeholder="example.toml" />
          </label>

          <div class="toolbar-meta">
            <span class="inline-pill" :class="{ ok: !isDirty }">
              {{ isDirty ? '有未保存改动' : '已同步' }}
            </span>
            <span class="inline-pill" :class="{ ok: app.info.value.current_config_file === normalizedFileName }">
              {{ app.info.value.current_config_file === normalizedFileName ? '当前运行配置' : '未运行' }}
            </span>
            <span class="inline-pill" :class="{ ok: editorMode === 'visual' }">
              {{ editorMode === 'visual' ? '图形模式' : '文件模式' }}
            </span>
          </div>
        </div>

        <div class="action-bar">
          <button
            class="button ghost"
            :disabled="app.loading.value || (app.info.value.status !== 'running' && app.startStatus.value !== 'starting')"
            @click="app.stop()"
          >
            停止
          </button>
          <button class="button ghost" :disabled="loadingEditor" @click="removeCurrentConfig()">删除</button>
          <button class="button ghost" :disabled="loadingEditor" @click="saveCurrentConfig()">保存</button>
          <button
            class="button ghost"
            :disabled="app.loading.value || app.startStatus.value === 'starting' || app.info.value.status !== 'running'"
            @click="restartCurrent()"
          >
            重启
          </button>
          <button
            class="button primary"
            :disabled="app.loading.value || app.startStatus.value === 'starting' || app.info.value.status === 'running'"
            @click="startCurrent()"
          >
            启动
          </button>
        </div>

        <div class="mode-note">
          <span v-if="editorMode === 'visual'">
            图形化模式会生成标准化 TOML，适合日常维护和减少手写错误。
          </span>
          <span v-else>
            文件模式会保留原始 TOML 文本，切回图形模式时会尝试重新解析。
          </span>
        </div>
      </SectionCard>

      <SectionCard
        v-if="editorMode === 'visual'"
        title="图形化配置"
        subtitle="常用字段全部图形化，复杂项保留可直接编辑的文本或列表。"
      >
        <div class="form-grid">
          <label class="field">
            <span>显示名称</span>
            <input v-model="visualConfig.config_name" type="text" placeholder="default" />
          </label>
          <label class="field">
            <span>网络代码</span>
            <input v-model="visualConfig.network_code" type="text" placeholder="default" />
          </label>
          <label class="field">
            <span>网络密钥</span>
            <input
              v-model="visualConfig.network_secret"
              type="text"
              placeholder="replace_with_the_server_side_network_secret"
            />
          </label>
          <label class="field">
            <span>证书校验</span>
            <select v-model="visualConfig.cert_mode">
              <option value="skip">skip</option>
              <option value="standard">standard</option>
              <option value="finger:">finger:&lt;sha256&gt;</option>
            </select>
          </label>
          <label class="field">
            <span>设备名称</span>
            <input v-model="visualConfig.device_name" type="text" placeholder="desktop-node" />
          </label>
          <label class="field">
            <span>设备 ID</span>
            <input v-model="visualConfig.device_id" type="text" placeholder="留空时自动生成" />
          </label>
          <label class="field">
            <span>TUN 名称</span>
            <input v-model="visualConfig.tun_name" type="text" placeholder="vnt0" />
          </label>
          <label class="field">
            <span>固定虚拟 IP</span>
            <input v-model="visualConfig.ip" type="text" placeholder="172.16.57.10" />
          </label>
          <label class="field">
            <span>数据密码</span>
            <input v-model="visualConfig.password" type="text" placeholder="可选的数据面加密密码" />
          </label>
          <label class="field">
            <span>MTU</span>
            <input v-model="visualConfig.mtu" type="text" inputmode="numeric" placeholder="1380" />
          </label>
          <label class="field">
            <span>隧道端口</span>
            <input v-model="visualConfig.tunnel_port" type="text" inputmode="numeric" placeholder="30001" />
          </label>
        </div>

        <div class="multi-grid">
          <label class="field">
            <span>控制服务器</span>
            <textarea
              v-model="serverText"
              rows="5"
              spellcheck="false"
              placeholder="每行一个地址，例如&#10;quic://1.2.3.4:29872&#10;tcp://1.2.3.4:6660"
            />
          </label>
          <label class="field">
            <span>允许出口路由</span>
            <textarea
              v-model="outputText"
              rows="5"
              spellcheck="false"
              placeholder="每行一个网段，例如&#10;0.0.0.0/0"
            />
          </label>
          <label class="field">
            <span>端口映射</span>
            <textarea
              v-model="portMappingText"
              rows="5"
              spellcheck="false"
              placeholder="每行一个规则，例如&#10;tcp://0.0.0.0:8080-172.16.57.20-172.16.57.20:80"
            />
          </label>
          <label class="field">
            <span>UDP STUN</span>
            <textarea
              v-model="udpStunText"
              rows="5"
              spellcheck="false"
              placeholder="每行一个 STUN 服务器"
            />
          </label>
          <label class="field">
            <span>TCP STUN</span>
            <textarea
              v-model="tcpStunText"
              rows="5"
              spellcheck="false"
              placeholder="每行一个 STUN 服务器"
            />
          </label>
        </div>

        <div class="toggle-grid">
          <label class="toggle-item">
            <span>压缩</span>
            <input v-model="visualConfig.compress" type="checkbox" />
          </label>
          <label class="toggle-item">
            <span>重传</span>
            <input v-model="visualConfig.rtx" type="checkbox" />
          </label>
          <label class="toggle-item">
            <span>FEC</span>
            <input v-model="visualConfig.fec" type="checkbox" />
          </label>
          <label class="toggle-item">
            <span>禁用打洞</span>
            <input v-model="visualConfig.no_punch" type="checkbox" />
          </label>
          <label class="toggle-item">
            <span>禁用 NAT</span>
            <input v-model="visualConfig.no_nat" type="checkbox" />
          </label>
          <label class="toggle-item">
            <span>禁用 TUN</span>
            <input v-model="visualConfig.no_tun" type="checkbox" />
          </label>
          <label class="toggle-item">
            <span>允许被映射</span>
            <input v-model="visualConfig.allow_mapping" type="checkbox" />
          </label>
        </div>

        <div class="route-head">
          <div>
            <h3>子网转发</h3>
            <p>每条规则由“目标网段 + 目标节点虚拟 IP”组成。</p>
          </div>
          <button class="button ghost" @click="addInputRule()">新增规则</button>
        </div>

        <div v-if="visualConfig.input.length === 0" class="empty-state">
          当前没有子网转发规则，需要时再添加即可。
        </div>

        <div v-else class="route-list">
          <div v-for="(item, index) in visualConfig.input" :key="index" class="route-item">
            <label class="field">
              <span>网段</span>
              <input v-model="item.net" type="text" placeholder="192.168.10.0/24" />
            </label>
            <label class="field">
              <span>目标 IP</span>
              <input v-model="item.target_ip" type="text" placeholder="172.16.57.10" />
            </label>
            <button class="button ghost" @click="removeInputRule(index)">删除</button>
          </div>
        </div>

        <div class="preview-card">
          <div class="preview-head">
            <div>
              <h3>生成的 TOML 预览</h3>
              <p>保存和启动时使用这一份标准化输出。</p>
            </div>
            <button class="button ghost" @click="updateRawFromVisual()">同步到文件模式</button>
          </div>
          <pre class="preview-code">{{ generatedToml }}</pre>
        </div>
      </SectionCard>

      <SectionCard
        v-else
        title="文件配置器"
        subtitle="直接编辑原始 TOML，适合手工调试、复制粘贴和精细修改。"
      >
        <div class="inline-actions">
          <button class="button ghost" @click="updateVisualFromRaw()">解析到图形模式</button>
          <button class="button ghost" @click="updateRawFromVisual()">用图形配置覆盖文件</button>
        </div>

        <div v-if="parseError" class="error-box">
          <strong>TOML 解析失败</strong>
          <span>{{ parseError }}</span>
        </div>

        <textarea
          v-model="editorContent"
          class="editor"
          spellcheck="false"
          placeholder="在这里直接编辑 TOML 配置"
        />

        <div class="editor-foot">
          <span>{{ loadingEditor ? '处理中...' : `${editorContent.length} chars` }}</span>
          <span>{{ normalizedFileName || '未命名配置' }}</span>
        </div>
      </SectionCard>
    </div>
  </div>
</template>

<style scoped>
.config-layout {
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(260px, 320px) minmax(0, 1fr);
}

.editor-stack {
  display: grid;
  gap: 1rem;
}

.inline-actions,
.action-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
}

.config-list {
  display: flex;
  max-height: 38rem;
  flex-direction: column;
  gap: 0.75rem;
  overflow: auto;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.02);
  padding: 0.95rem 1rem;
  color: inherit;
  text-align: left;
}

.config-item span {
  color: var(--text-soft);
  font-size: 0.88rem;
}

.config-item.active {
  border-color: rgba(124, 155, 255, 0.3);
  background: rgba(124, 155, 255, 0.08);
}

.editor-toolbar,
.editor-foot,
.preview-head,
.route-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.mode-note {
  color: var(--text-soft);
  font-size: 0.92rem;
}

.toolbar-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
}

.form-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.multi-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.field {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.45rem;
}

.field span {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.field textarea,
.editor,
.preview-code {
  min-height: 7rem;
  border: 1px solid var(--border-strong);
  border-radius: 1rem;
  background: rgba(4, 11, 20, 0.92);
  padding: 1rem;
  color: #d7e6fb;
  font-family: 'IBM Plex Mono', 'Cascadia Code', 'Consolas', monospace;
  font-size: 0.93rem;
  line-height: 1.65;
}

.field select {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 0.9rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 0.8rem 0.95rem;
  color: var(--text-main);
}

.toggle-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.toggle-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  border-radius: 0.95rem;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(255, 255, 255, 0.025);
  padding: 0.9rem 1rem;
}

.toggle-item span {
  color: var(--text-soft);
}

.route-head {
  align-items: flex-end;
}

.route-head h3,
.preview-head h3 {
  margin: 0;
}

.route-head p,
.preview-head p {
  margin: 0.35rem 0 0;
  color: var(--text-soft);
  font-size: 0.9rem;
}

.route-list {
  display: grid;
  gap: 0.75rem;
}

.route-item {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  align-items: end;
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.025);
  padding: 0.95rem;
}

.preview-card {
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 1.25rem;
  background: rgba(255, 255, 255, 0.025);
  padding: 1rem;
}

.preview-code {
  margin: 1rem 0 0;
  overflow: auto;
  white-space: pre-wrap;
}

.editor {
  min-height: 36rem;
  resize: vertical;
}

.editor-foot {
  color: var(--text-soft);
  font-size: 0.9rem;
}

.error-box {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  border: 1px solid rgba(239, 90, 90, 0.28);
  border-radius: 1rem;
  background: rgba(87, 20, 20, 0.28);
  padding: 0.95rem 1rem;
  color: #ffb0b0;
}

@media (max-width: 1200px) {
  .config-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 880px) {
  .form-grid,
  .multi-grid,
  .toggle-grid,
  .route-item {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .editor-toolbar,
  .editor-foot,
  .preview-head,
  .route-head {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
