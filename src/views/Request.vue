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
        <el-form :model="newRequest" label-width="120px" style="max-width: 500px; margin: 0 auto;">
          <el-form-item label="充电模式">
            <el-radio-group v-model="newRequest.mode">
              <el-radio label="Fast">快充 (30度/小时)</el-radio>
              <el-radio label="Slow">慢充 (7度/小时)</el-radio>
            </el-radio-group>
          </el-form-item>
          <el-form-item label="充电量(度)">
            <el-input-number v-model="newRequest.amount" :min="1" :max="100" :step="1" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="createRequest" :loading="loading" style="width: 120px;">提交请求</el-button>
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
            <el-descriptions-item label="排队号">{{ currentRequest.queue_number }}</el-descriptions-item>
            <el-descriptions-item label="充电模式">
              <el-tag :type="currentRequest.mode === 'Fast' ? 'danger' : 'success'">
                {{ currentRequest.mode === 'Fast' ? '快充' : '慢充' }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="充电量">{{ currentRequest.amount }}度</el-descriptions-item>
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
              <el-switch
                v-model="autoRefresh"
                active-text="自动刷新"
                style="margin-right: 10px;"
              />
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
                  <el-table-column prop="amount" label="充电量" width="80" />
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
                  <el-table-column prop="amount" label="充电量" width="80" />
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
      loading: false,
      user: null,
      autoRefresh: false,
      refreshInterval: null,
      
      // 新建请求表单
      newRequest: {
        mode: 'Fast',
        amount: 30
      },
      
      // 表单验证规则
      rules: {
        mode: [
          { required: true, message: '请选择充电模式', trigger: 'change' }
        ],
        amount: [
          { required: true, message: '请输入充电量', trigger: 'blur' },
          { type: 'number', min: 1, max: 100, message: '充电量必须在1-100度之间', trigger: 'blur' }
        ]
      },
      
      // 当前请求
      currentRequest: null,
      
      // 充电队列
      fastQueue: [],
      slowQueue: [],
      
      // 修改请求对话框
      editDialogVisible: false,
      editForm: {
        id: '',
        mode: 'Fast',
        amount: 30,
        originalMode: ''
      }
    }
  },
  
  computed: {
    hasActiveRequest() {
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
  },

  beforeUnmount() {
    // 清除自动刷新定时器
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval)
    }
  },
  
  watch: {
    autoRefresh(newVal) {
      if (newVal) {
        this.refreshInterval = setInterval(() => {
          this.loadCurrentRequest()
          this.loadQueues()
        }, 5000) // 每5秒刷新一次
      } else {
        if (this.refreshInterval) {
          clearInterval(this.refreshInterval)
          this.refreshInterval = null
        }
      }
    }
  },
  
  methods: {
    // 加载当前用户的充电请求
    async loadCurrentRequest() {
      if (!this.user) {
        ElMessage.warning('请先登录')
        return
      }
      
      this.loading = true
      try {
        const response = await axios.get(`http://localhost:8080/users/${this.user.id}/charging-requests`)
        
        if (response.data.success) {
          const requests = response.data.data
          // 找到未完成的请求
          this.currentRequest = requests.find(r => 
            r.status !== 'Completed' && r.status !== 'Cancelled'
          ) || null
        } else {
          ElMessage.error(response.data.message || '加载请求失败')
        }
      } catch (error) {
        console.error('加载请求失败:', error)
        ElMessage.error(error.response?.data?.message || '加载请求失败，请稍后重试')
      } finally {
        this.loading = false
      }
    },
    
    // 创建充电请求
    async createRequest() {
      if (!this.user) return
      
      this.loading = true
      try {
        const response = await axios.post('http://localhost:8080/charging-requests', {
          user_id: this.user.id,
          mode: this.newRequest.mode,
          amount: this.newRequest.amount
        })
        
        if (response.data.success) {
          ElMessage.success('充电请求创建成功！')
          this.newRequest = { mode: 'Fast', amount: 30 }
          this.loadCurrentRequest()
          this.loadQueues()
        } else {
          ElMessage.error(response.data.message || '创建失败')
        }
      } catch (error) {
        console.error('创建请求失败:', error)
        ElMessage.error(error.response?.data?.message || '创建请求失败，请稍后重试')
      } finally {
        this.loading = false
      }
    },
    
    // 加载充电队列
    async loadQueues() {
      try {
        const [fastResponse, slowResponse] = await Promise.all([
          axios.get('http://localhost:8080/charging-requests/queue/fast'),
          axios.get('http://localhost:8080/charging-requests/queue/slow')
        ])
        
        if (fastResponse.data.success) {
          // 确保快充队列按创建时间排序
          this.fastQueue = fastResponse.data.data.sort((a, b) => 
            new Date(a.created_at) - new Date(b.created_at)
          );
        }
        if (slowResponse.data.success) {
          // 确保慢充队列按创建时间排序
          this.slowQueue = slowResponse.data.data.sort((a, b) => 
            new Date(a.created_at) - new Date(b.created_at)
          );
        }
      } catch (error) {
        console.error('加载队列失败:', error)
        ElMessage.error('加载队列失败，请稍后重试')
      }
    },
    
    // 编辑请求
    editRequest(request) {
      this.editForm = {
        id: request.id,
        mode: request.mode,
        amount: request.amount,
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
        // 如果模式改变了，需要调用修改模式接口
        if (this.editForm.mode !== this.editForm.originalMode) {
          await axios.put(`http://localhost:8080/charging-requests/${this.editForm.id}/mode`, {
            mode: this.editForm.mode,
            queue_number: ''
          })
        }
        
        // 如果充电量改变了，需要调用修改充电量接口
        if (this.editForm.amount !== this.currentRequest.amount) {
          await axios.put(`http://localhost:8080/charging-requests/${this.editForm.id}/amount`, {
            amount: this.editForm.amount
          })
        }
        
        ElMessage.success('修改成功！')
        this.editDialogVisible = false
        this.loadCurrentRequest()
        this.loadQueues()
      } catch (error) {
        console.error('修改失败:', error)
        ElMessage.error(error.response?.data?.message || '修改失败，请稍后重试')
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
        const response = await axios.delete(`http://localhost:8080/charging-requests/${requestId}`)
        
        if (response.data.success) {
          ElMessage.success('请求已取消')
          this.loadCurrentRequest()
          this.loadQueues()
        } else {
          ElMessage.error(response.data.message || '取消失败')
        }
      } catch (error) {
        if (error !== 'cancel') {
          console.error('取消请求失败:', error)
          ElMessage.error(error.response?.data?.message || '取消请求失败，请稍后重试')
        }
      } finally {
        this.loading = false
      }
    },
    
    // 获取状态标签类型
    getStatusType(status) {
      switch (status) {
        case 'Waiting': return 'warning'
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
        case 'Charging': return '充电中'
        case 'Completed': return '已完成'
        case 'Cancelled': return '已取消'
        default: return status
      }
    },
    
    // 格式化日期
    formatDate(dateString) {
      return new Date(dateString).toLocaleString('zh-CN')
    },
    
    // 返回主页
    goBack() {
      this.$router.push('/main')
    }
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