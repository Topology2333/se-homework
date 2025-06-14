<template>
  <el-container style="width: 90vw; margin: 0 auto; padding: 20px; background: transparent; background-color: rgba(255, 255, 255, 0.9);">
    <el-header style="font-size: 28px; font-weight: bold; text-align: center; margin-bottom: 30px; color: #409EFF;">
      充电请求管理
    </el-header>

    <el-main>
      <!-- 创建充电请求表单 -->
      <el-card v-if="!hasActiveRequest" shadow="hover" style="margin-bottom: 20px; border-radius: 8px;">
        <template #header>
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <h3 style="margin: 0;">新建充电请求</h3>
            <el-button type="primary" @click="goBack" plain>返回主页</el-button>
          </div>
        </template>
        <el-form :model="requestForm" label-width="120px" style="max-width: 500px; margin: 0 auto;">
          <el-form-item label="充电模式">
            <el-radio-group v-model="requestForm.mode">
              <el-radio label="Fast">快充 (30度/小时)</el-radio>
              <el-radio label="Slow">慢充 (7度/小时)</el-radio>
            </el-radio-group>
          </el-form-item>
          <el-form-item label="充电量(度)">
            <el-input-number v-model="requestForm.amount" :min="1" :max="100" :step="1" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="submitRequest" :loading="loading" style="width: 120px;">提交请求</el-button>
          </el-form-item>
        </el-form>
      </el-card>

      <!-- 当前充电请求信息 -->
      <template v-if="hasActiveRequest">
        <el-card shadow="hover" style="margin-bottom: 20px; border-radius: 8px;">
          <template #header>
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <h3 style="margin: 0;">当前充电请求</h3>
              <div>
                <el-button type="primary" @click="goBack" plain>返回主页</el-button>
              </div>
            </div>
          </template>
          
          <el-descriptions :column="2" border>
            <el-descriptions-item label="充电桩编号">
              <el-tag type="info">{{ currentRequest.pile_number || '未分配' }}</el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="排队号">{{ currentRequest.queue_number }}</el-descriptions-item>
            <el-descriptions-item label="充电模式">
              <el-tag :type="currentRequest.mode === 'Fast' ? 'danger' : 'success'">
                {{ currentRequest.mode === 'Fast' ? '快充' : '慢充' }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="请求充电量">{{ currentRequest.requested_amount }}度</el-descriptions-item>
            <el-descriptions-item label="充电进度">{{ formatProgress(currentRequest.amount) }}%</el-descriptions-item>
            <el-descriptions-item label="前面等待车辆">
              <el-tag type="warning">{{ currentRequest.waiting_count || 0 }}辆</el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="状态">
              <el-tag :type="getStatusType(currentRequest.status)">
                {{ getStatusText(currentRequest.status) }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="创建时间" :span="2">
              {{ formatDate(currentRequest.created_at) }}
            </el-descriptions-item>
          </el-descriptions>

          <div style="margin-top: 20px; display: flex; gap: 10px; justify-content: center;">
            <el-button 
              type="primary" 
              @click="editRequest(currentRequest)"
              :disabled="currentRequest.status !== 'Waiting'"
            >
              修改请求
            </el-button>
            <el-button 
              type="danger" 
              @click="cancelRequest(currentRequest.id)"
              :disabled="currentRequest.status === 'Completed' || currentRequest.status === 'Cancelled'"
            >
              取消请求
            </el-button>
          </div>
        </el-card>

        <!-- 排队信息 -->
        <el-card shadow="hover" style="border-radius: 8px;">
          <template #header>
                          <div style="display: flex; justify-content: space-between; align-items: center;">
                <h3 style="margin: 0;">排队信息</h3>
              </div>
          </template>

          <el-row :gutter="20">
            <el-col :span="12">
              <el-card shadow="hover" style="border-radius: 8px;">
                <template #header>
                  <div style="display: flex; justify-content: space-between; align-items: center;">
                    <h4 style="margin: 0;">快充队列</h4>
                    <el-tag type="danger">30度/小时</el-tag>
                  </div>
                </template>
                <el-table :data="fastQueue" style="width: 100%" size="small" border stripe>
                  <el-table-column prop="queue_number" label="排队号" width="80" />
                  <el-table-column label="充电桩编号" width="100">
                    <template #default="scope">
                      <el-tag type="info" size="small">{{ scope.row.pile_number || '待分配' }}</el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="充电进度" width="100">
                    <template #default="scope">
                      {{ formatProgress(scope.row.amount) }}%
                    </template>
                  </el-table-column>
                  <el-table-column prop="created_at" label="排队时间" min-width="120">
                    <template #default="scope">
                      {{ formatDate(scope.row.created_at) }}
                    </template>
                  </el-table-column>
                </el-table>
              </el-card>
            </el-col>
            <el-col :span="12">
              <el-card shadow="hover" style="border-radius: 8px;">
                <template #header>
                  <div style="display: flex; justify-content: space-between; align-items: center;">
                    <h4 style="margin: 0;">慢充队列</h4>
                    <el-tag type="success">7度/小时</el-tag>
                  </div>
                </template>
                <el-table :data="slowQueue" style="width: 100%" size="small" border stripe>
                  <el-table-column prop="queue_number" label="排队号" width="80" />
                  <el-table-column label="充电桩编号" width="100">
                    <template #default="scope">
                      <el-tag type="info" size="small">{{ scope.row.pile_number || '待分配' }}</el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="充电进度" width="100">
                    <template #default="scope">
                      {{ formatProgress(scope.row.amount) }}%
                    </template>
                  </el-table-column>
                  <el-table-column prop="created_at" label="排队时间" min-width="120">
                    <template #default="scope">
                      {{ formatDate(scope.row.created_at) }}
                    </template>
                  </el-table-column>
                </el-table>
              </el-card>
            </el-col>
          </el-row>
        </el-card>
      </template>
    </el-main>

    <!-- 修改请求对话框 -->
    <el-dialog 
      v-model="editDialogVisible" 
      title="修改充电请求" 
      width="500px" 
      destroy-on-close
      :close-on-click-modal="false"
      :close-on-press-escape="false"
    >
      <el-form :model="editForm" label-width="120px" :rules="rules" ref="editFormRef">
        <el-form-item label="充电模式" prop="mode">
          <el-radio-group v-model="editForm.mode">
            <el-radio label="Fast">快充 (30度/小时)</el-radio>
            <el-radio label="Slow">慢充 (7度/小时)</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="充电量(度)" prop="amount">
          <el-input-number v-model="editForm.amount" :min="1" :max="100" :step="1" />
        </el-form-item>
      </el-form>
      <div class="dialog-footer" style="margin-top: 30px; display: flex; justify-content: center; gap: 20px;">
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="updateRequest" :loading="loading" style="width: 120px;">确认修改</el-button>
      </div>
    </el-dialog>
  </el-container>
</template>

<script>
import axios from 'axios'
import { ElMessage, ElMessageBox } from 'element-plus'

export default {
  name: 'ChargingRequest',
  data() {
    return {
      user: null,
      loading: false,
      requestForm: {
        mode: 'Fast',
        amount: 30
      },
      currentRequest: null,
      fastQueue: [],
      slowQueue: [],
      autoRefresh: false,
      editDialogVisible: false,
      editForm: {
        id: '',
        mode: 'Fast',
        amount: 30,
        originalMode: ''
      },
      rules: {
        mode: [
          { required: true, message: '请选择充电模式', trigger: 'change' }
        ],
        amount: [
          { required: true, message: '请输入充电量', trigger: 'blur' },
          { type: 'number', min: 1, max: 100, message: '充电量必须在1-100度之间', trigger: 'blur' }
        ]
      },
      refreshTimer: null
    }
  },
  
  computed: {
    hasActiveRequest() {
      console.log('计算hasActiveRequest, 当前请求:', this.currentRequest)
      return this.currentRequest && 
             this.currentRequest.status !== 'Completed' && 
             this.currentRequest.status !== 'Cancelled'
    }
  },
  
  created() {
    // 获取用户信息
    const user = JSON.parse(localStorage.getItem('user'))
    if (!user) {
      this.$router.push('/home')
      return
    }
    this.user = user
    
    // 加载数据
    this.loadCurrentRequest()
    this.loadQueues()
    this.startAutoRefresh()
  },

  beforeUnmount() {
    // 清除自动刷新定时器
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer)
    }
  },
  
  watch: {
    autoRefresh(newVal) {
      if (newVal) {
        this.refreshTimer = setInterval(() => {
          this.loadCurrentRequest()
          this.loadQueues()
        }, 5000) // 每5秒刷新一次
      } else {
        if (this.refreshTimer) {
          clearInterval(this.refreshTimer)
          this.refreshTimer = null
        }
      }
    }
  },
  
  methods: {
    // 加载当前用户的充电请求
    async loadCurrentRequest() {
      try {
        console.log('=== 开始加载当前请求 ===');
        console.log('当前用户ID:', this.user.id);
        
        // 步骤 1: 从调度器获取系统实时状态，这是唯一且完全可靠的数据源
        const response = await axios.get('http://localhost:8080/api/scheduler/status');
        const systemStatus = response.data;
        console.log('获取到的系统状态:', systemStatus);
        
        if (!systemStatus) {
          console.error("获取系统实时状态失败。");
          this.currentRequest = null;
          return;
        }

        let userRequest = null;
        let userStatus = '';
        let waitingCount = 0;

        // 步骤 2: 在唯一的、可靠的数据源中查找用户的位置和信息
        console.log('开始检查充电桩状态...');
        for (const pile of systemStatus.pile_status || []) {
          console.log(`检查充电桩 ${pile.pile_number}:`, {
            current_request: pile.current_request,
            queue_requests: pile.queue_requests
          });
          
          if (pile.current_request && pile.current_request.user_id === this.user.id) {
            console.log(`找到用户请求在充电桩 ${pile.pile_number} 充电中`);
            userRequest = { ...pile.current_request, pile_number: pile.pile_number, progress: pile.charging_progress || 0 };
            userStatus = 'Charging';
            waitingCount = 0;
            break;
          }
          
          const queueIndex = pile.queue_requests?.findIndex(req => req.user_id === this.user.id) ?? -1;
          if (queueIndex !== -1) {
            console.log(`找到用户请求在充电桩 ${pile.pile_number} 排队中，位置: ${queueIndex + 1}`);
            userRequest = { ...pile.queue_requests[queueIndex], pile_number: pile.pile_number, progress: 0 };
            userStatus = 'Queued';
            waitingCount = 1 + queueIndex;
            break;
          }
        }

        // 如果不在充电桩，则检查等待区
        if (!userRequest) {
          console.log('未在充电桩找到请求，检查等待区...');
          console.log('快充等待区:', systemStatus.fast_waiting_requests);
          console.log('慢充等待区:', systemStatus.slow_waiting_requests);
          
          const fastWaitingIndex = systemStatus.fast_waiting_requests?.findIndex(req => req.user_id === this.user.id) ?? -1;
          if (fastWaitingIndex !== -1) {
            console.log(`找到用户请求在快充等待区，位置: ${fastWaitingIndex + 1}`);
            userRequest = { ...systemStatus.fast_waiting_requests[fastWaitingIndex], pile_number: '未分配', progress: 0 };
            userStatus = 'Waiting';
            const fastPiles = systemStatus.pile_status.filter(p => p.pile_mode === 'Fast');
            waitingCount = fastPiles.reduce((acc, p) => acc + (p.current_request ? 1 : 0) + (p.queue_requests?.length || 0), 0) + fastWaitingIndex;
          } else {
            const slowWaitingIndex = systemStatus.slow_waiting_requests?.findIndex(req => req.user_id === this.user.id) ?? -1;
            if (slowWaitingIndex !== -1) {
              console.log(`找到用户请求在慢充等待区，位置: ${slowWaitingIndex + 1}`);
              userRequest = { ...systemStatus.slow_waiting_requests[slowWaitingIndex], pile_number: '未分配', progress: 0 };
              userStatus = 'Waiting';
              const slowPiles = systemStatus.pile_status.filter(p => p.pile_mode === 'Slow');
              waitingCount = slowPiles.reduce((acc, p) => acc + (p.current_request ? 1 : 0) + (p.queue_requests?.length || 0), 0) + slowWaitingIndex;
            }
          }
        }
        
        // 步骤 3: 根据找到的唯一直实信息更新UI
        if (userRequest) {
          console.log('找到用户请求，准备更新UI:', userRequest);
          this.currentRequest = {
            id: userRequest.id,
            user_id: userRequest.user_id,
            mode: userRequest.mode,
            status: userStatus,
            requested_amount: userRequest.amount,
            amount: userRequest.progress || 0,
            pile_number: userRequest.pile_number || '未分配',
            queue_number: userRequest.queue_number,
            created_at: new Date(userRequest.created_at),
            waiting_count: waitingCount,
          };
          console.log('更新后的currentRequest:', this.currentRequest);
        } else {
          console.log('未找到用户请求，设置currentRequest为null');
          this.currentRequest = null;
        }
        
        console.log('=== 加载当前请求完成 ===');
      } catch (error) {
        console.error('加载当前充电请求时发生严重错误:', error);
        console.error('错误详情:', {
          message: error.message,
          response: error.response?.data,
          status: error.response?.status
        });
        this.currentRequest = null;
      }
    },
    
    // 创建充电请求
    async submitRequest() {
      if (!this.user) {
        ElMessage.warning('请先登录')
        return
      }

      try {
        this.loading = true
        
        // 保存用户请求的数据，防止后续被覆盖
        const requestedAmount = this.requestForm.amount
        const requestedMode = this.requestForm.mode
        
        console.log('提交请求数据:', {
          user_id: this.user.id,
          mode: requestedMode,
          amount: requestedAmount
        })
        
        const response = await axios.post('http://localhost:8080/api/scheduler/submit', {
          user_id: this.user.id,
          mode: requestedMode,
          amount: requestedAmount
        })

        console.log('提交请求响应:', response.data)

        if (response.data) {
          ElMessage.success('充电请求已提交')
          
          // 使用后端返回的真实请求数据
          this.currentRequest = {
            id: response.data.id || response.data.request_id, // 使用后端返回的真实ID
            user_id: this.user.id,
            mode: requestedMode,
            amount: 0, // 充电进度初始为0
            requested_amount: requestedAmount, // 用户请求的充电量
            pile_number: '未分配', // 充电桩编号，提交时未分配
            queue_number: response.data.queue_number || response.data.queue_num || '待分配', // 使用后端返回的排队号
            status: response.data.status || 'Waiting',
            created_at: response.data.created_at ? new Date(response.data.created_at) : new Date(),
            waiting_count: response.data.waiting_count || 0 // 初始等待车辆数
          }
          
          console.log('提交后立即设置的当前请求:', this.currentRequest)
          console.log('hasActiveRequest:', this.hasActiveRequest)
          
          // 重置表单
          this.requestForm = {
            mode: 'Fast',
            amount: 30
          }
          
          // 延迟刷新，获取真实数据
          setTimeout(async () => {
            console.log('开始延迟加载最新数据...')
            await this.loadCurrentRequest()
            await this.loadQueues()
            console.log('延迟数据加载完成, 当前请求:', this.currentRequest)
          }, 1000)
        }
      } catch (error) {
        console.error('提交请求失败:', error)
        console.error('错误详情:', {
          message: error.message,
          response: error.response?.data,
          status: error.response?.status
        })
        ElMessage.error(error.response?.data?.message || '提交请求失败，请稍后重试')
      } finally {
        this.loading = false
      }
    },
    
    // 加载充电队列
    async loadQueues() {
      try {
        const response = await axios.get('http://localhost:8080/api/scheduler/status')
        const status = response.data
        console.log('加载队列数据:', status)
        
        // 收集所有快充和慢充的请求
        let allFastRequests = []
        let allSlowRequests = []
        
        for (const pile of status.pile_status) {
          console.log('处理充电桩:', pile.pile_number, pile)
          
          // 添加当前充电的请求
          if (pile.current_request) {
            const requestWithPile = {
              ...pile.current_request,
              pile_number: pile.pile_number,
              charging_progress: pile.charging_progress || 0,
              status: 'Charging' // 强制设置为充电中
            }
            
            if (pile.pile_mode === 'Fast') {
              allFastRequests.push(requestWithPile)
            } else if (pile.pile_mode === 'Slow') {
              allSlowRequests.push(requestWithPile)
            }
          }
          
          // 添加队列中的请求
          if (pile.queue_requests && pile.queue_requests.length > 0) {
            for (const queueRequest of pile.queue_requests) {
              const requestWithPile = {
                ...queueRequest,
                pile_number: pile.pile_number,
                charging_progress: 0,
                status: 'Waiting' // 强制设置为等待中
              }
              
              if (pile.pile_mode === 'Fast') {
                allFastRequests.push(requestWithPile)
              } else if (pile.pile_mode === 'Slow') {
                allSlowRequests.push(requestWithPile)
              }
            }
          }
        }
        
        // 转换为前端需要的格式
        this.fastQueue = allFastRequests.map(req => ({
          queue_number: req.queue_number,
          pile_number: req.pile_number,
          amount: req.charging_progress,
          created_at: new Date(req.created_at)
        }))
        
        this.slowQueue = allSlowRequests.map(req => ({
          queue_number: req.queue_number,
          pile_number: req.pile_number,
          amount: req.charging_progress,
          created_at: new Date(req.created_at)
        }))
        
        console.log('快充队列:', this.fastQueue)
        console.log('慢充队列:', this.slowQueue)
      } catch (error) {
        console.error('加载充电队列失败:', error)
        this.fastQueue = []
        this.slowQueue = []
        ElMessage.error('加载队列信息失败，请稍后重试')
      }
    },
    
    // 编辑请求
    editRequest(request) {
      this.editForm = {
        id: request.id,
        mode: request.mode,
        amount: request.requested_amount || request.amount, // 使用请求量而不是进度
        originalMode: request.mode
      }
      this.editDialogVisible = true
    },
    
    // 更新请求
    async updateRequest() {
      if (!this.$refs.editFormRef) return
      
      try {
        await this.$refs.editFormRef.validate()
      } catch (error) {
        return
      }
      
      this.loading = true
      try {
        // 检查是否修改了模式或充电量
        const modeChanged = this.editForm.mode !== this.editForm.originalMode
        const amountChanged = this.editForm.amount !== (this.currentRequest.requested_amount || this.currentRequest.amount)
        
        let response
        
        if (modeChanged && amountChanged) {
          // 如果同时修改了模式和充电量，先修改模式（会重新排队），再修改充电量
          console.log('同时修改模式和充电量')
          
          // 先修改模式
          const modeResponse = await axios.put(`http://localhost:8080/api/charging-requests/${this.editForm.id}/mode`, {
            mode: this.editForm.mode,
            queue_number: this.currentRequest.queue_number // 新增，保证后端不报错
          })
          
          if (modeResponse.data && modeResponse.data.success) {
            // 再修改充电量
            response = await axios.put(`http://localhost:8080/api/charging-requests/${this.editForm.id}/amount`, {
              amount: this.editForm.amount
            })
          } else {
            throw new Error(modeResponse.data?.message || '修改模式失败')
          }
        } else if (modeChanged) {
          // 只修改模式
          console.log('只修改模式')
          response = await axios.put(`http://localhost:8080/api/charging-requests/${this.editForm.id}/mode`, {
            mode: this.editForm.mode,
            queue_number: this.currentRequest.queue_number // 新增，保证后端不报错
          })
        } else if (amountChanged) {
          // 只修改充电量
          console.log('只修改充电量')
          response = await axios.put(`http://localhost:8080/api/charging-requests/${this.editForm.id}/amount`, {
            amount: this.editForm.amount
          })
        } else {
          // 没有修改
          ElMessage.info('没有检测到修改')
          this.editDialogVisible = false
          this.loading = false
          return
        }
        
        if (response.data && response.data.success) {
          ElMessage.success('修改成功！')
          this.editDialogVisible = false
          
          // 更新本地数据
          this.currentRequest = {
            ...this.currentRequest,
            mode: this.editForm.mode,
            requested_amount: this.editForm.amount,
            queue_number: response.data.data?.queue_number || this.currentRequest.queue_number
          }
          
          // 延迟刷新数据，确保后端数据同步完成
          setTimeout(async () => {
            await this.loadCurrentRequest()
            await this.loadQueues()
          }, 500)
        } else {
          throw new Error(response.data?.message || '修改失败')
        }
      } catch (error) {
        console.error('修改失败:', error)
        ElMessage.error(error.response?.data?.message || error.message || '修改失败，请稍后重试')
      } finally {
        this.loading = false
      }
    },
    
    // 取消请求
    async cancelRequest(requestId) {
      try {
        await ElMessageBox.confirm('确定要取消这个充电请求吗？', '确认取消', {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning'
        })
        
        this.loading = true
        console.log('取消请求ID:', requestId)
        console.log('请求ID类型:', typeof requestId)
        console.log('请求ID是否为UUID格式:', /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(requestId))
        console.log('当前请求对象:', this.currentRequest)
        
        let response
        try {
          // 首先尝试使用请求ID
          console.log('尝试使用请求ID取消:', `http://localhost:8080/api/scheduler/cancel/${requestId}`)
          response = await axios.post(`http://localhost:8080/api/scheduler/cancel/${requestId}`)
        } catch (error) {
          console.log('使用请求ID取消失败，尝试使用用户ID:', error.message)
          console.log('错误状态码:', error.response?.status)
          console.log('错误响应:', error.response?.data)
          
          // 如果失败，尝试使用用户ID
          console.log('尝试使用用户ID取消:', `http://localhost:8080/api/scheduler/cancel/user/${this.user.id}`)
          response = await axios.post(`http://localhost:8080/api/scheduler/cancel/user/${this.user.id}`)
        }
        
        console.log('取消请求响应:', response.data)
        
        if (response.data && response.data.success !== false) {
          ElMessage.success('请求已取消')
          this.currentRequest = null
          await this.loadQueues()
        } else {
          throw new Error(response.data?.message || '取消请求失败')
        }
      } catch (error) {
        if (error !== 'cancel') {
          console.error('取消请求失败:', error)
          console.error('错误详情:', {
            message: error.message,
            response: error.response?.data,
            status: error.response?.status
          })
          ElMessage.error(error.response?.data?.message || error.message || '取消请求失败，请稍后重试')
        }
      } finally {
        this.loading = false
      }
    },
    
    // 格式化充电进度，保留两位小数
    formatProgress(value) {
      if (!value && value !== 0) return '0.00'
      return parseFloat(value).toFixed(2)
    },
    
    // 获取状态标签类型
    getStatusType(status) {
      switch (status) {
        case 'Waiting': return 'warning'
        case 'Queued': return 'info'
        case 'Charging': return 'primary'
        case 'Completed': return 'success'
        case 'Cancelled': return 'danger'
        default: return 'info'
      }
    },
    
    // 获取状态文本
    getStatusText(status) {
      switch (status) {
        case 'Waiting': return '等待中'
        case 'Queued': return '排队中'
        case 'Charging': return '充电中'
        case 'Completed': return '已完成'
        case 'Cancelled': return '已取消'
        default: return status
      }
    },
    
    // 格式化日期
    formatDate(date) {
      if (!date) return '--'
      const d = new Date(date)
      return d.toLocaleString('zh-CN')
    },
    
    // 返回主页
    goBack() {
      this.$router.push('/main')
    },

    // 启动自动刷新
    startAutoRefresh() {
      if (this.autoRefresh) {
        this.refreshTimer = setInterval(() => {
          this.loadCurrentRequest()
          this.loadQueues()
        }, 5000)
      }
    },
  }
}
</script>

<style scoped>
h3, h4 {
  margin-bottom: 15px;
  color: #303133;
}

.el-card {
  border-radius: 8px;
  transition: all 0.3s;
}

.el-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.dialog-footer {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 20px;
  margin-top: 20px;
  padding: 20px 0 0 0;
  min-height: 60px;
  background: #fff;
  z-index: 10;
}

:deep(.el-dialog__footer) {
  padding: 20px;
  border-top: 1px solid #e4e7ed;
}

:deep(.el-dialog__body) {
  padding: 20px;
}

.el-table {
  border-radius: 4px;
  overflow: hidden;
}

.el-button-group {
  display: flex;
  gap: 8px;
}

.el-tag {
  margin-right: 8px;
}

.el-descriptions {
  margin: 20px 0;
}
</style>