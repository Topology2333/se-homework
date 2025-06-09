<template>
  <el-container style="width: 70vw; margin: 0 auto; padding: 20px; background: transparent; background-color: rgba(255, 255, 255, 0.8);">
    <el-header style="font-size: 24px; font-weight: bold; text-align: center; margin-bottom: 20px;">
      充电请求管理
    </el-header>

    <el-main>
      <el-tabs v-model="activeTab" type="card">
        <!-- 创建新请求 -->
        <el-tab-pane label="创建充电请求" name="create">
          <el-card shadow="hover" style="margin-bottom: 20px;">
            <h3>新建充电请求</h3>
            <el-form :model="newRequest" label-width="120px" style="max-width: 500px;">
              <el-form-item label="充电模式">
                <el-radio-group v-model="newRequest.mode">
                  <el-radio label="Fast">快充 (30度/小时)</el-radio>
                  <el-radio label="Slow">慢充 (7度/小时)</el-radio>
                </el-radio-group>
              </el-form-item>
              <el-form-item label="充电量(度)">
                <el-input-number v-model="newRequest.amount" :min="1" :max="100" />
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="createRequest" :loading="loading">提交请求</el-button>
                <el-button @click="goBack">返回</el-button>
              </el-form-item>
            </el-form>
          </el-card>
        </el-tab-pane>

        <!-- 我的请求 -->
        <el-tab-pane label="我的请求" name="my-requests">
          <el-card shadow="hover" style="margin-bottom: 20px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
              <h3>我的充电请求</h3>
              <el-button @click="loadMyRequests" icon="Refresh" circle></el-button>
            </div>
            
            <el-table :data="myRequests" style="width: 100%" v-loading="loading">
              <el-table-column prop="queue_number" label="排队号" width="100" />
              <el-table-column prop="mode" label="充电模式" width="100">
                <template #default="scope">
                  {{ scope.row.mode === 'Fast' ? '快充' : '慢充' }}
                </template>
              </el-table-column>
              <el-table-column prop="amount" label="充电量(度)" width="120" />
              <el-table-column prop="status" label="状态" width="100">
                <template #default="scope">
                  <el-tag :type="getStatusType(scope.row.status)">
                    {{ getStatusText(scope.row.status) }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="created_at" label="创建时间" width="180">
                <template #default="scope">
                  {{ formatDate(scope.row.created_at) }}
                </template>
              </el-table-column>
              <el-table-column label="操作" width="250">
                <template #default="scope">
                  <el-button-group>
                    <el-button 
                      size="small" 
                      @click="editRequest(scope.row)"
                      :disabled="scope.row.status !== 'Waiting'"
                      type="primary"
                    >
                      修改
                    </el-button>
                    <el-button 
                      size="small" 
                      @click="cancelRequest(scope.row.id)"
                      :disabled="scope.row.status === 'Completed' || scope.row.status === 'Cancelled'"
                      type="danger"
                    >
                      取消
                    </el-button>
                  </el-button-group>
                </template>
              </el-table-column>
            </el-table>
          </el-card>
        </el-tab-pane>

        <!-- 充电队列 -->
        <el-tab-pane label="充电队列" name="queue">
          <el-row :gutter="20">
            <el-col :span="12">
              <el-card shadow="hover">
                <h3>快充队列</h3>
                <el-table :data="fastQueue" style="width: 100%" size="small">
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
              <el-card shadow="hover">
                <h3>慢充队列</h3>
                <el-table :data="slowQueue" style="width: 100%" size="small">
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
        </el-tab-pane>
      </el-tabs>
    </el-main>

    <!-- 修改请求对话框 -->
    <el-dialog v-model="editDialogVisible" title="修改充电请求" width="500px">
      <el-form :model="editForm" label-width="120px">
        <el-form-item label="充电模式">
          <el-radio-group v-model="editForm.mode">
            <el-radio label="Fast">快充 (30度/小时)</el-radio>
            <el-radio label="Slow">慢充 (7度/小时)</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="充电量(度)">
          <el-input-number v-model="editForm.amount" :min="1" :max="100" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="editDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="updateRequest" :loading="loading">确定</el-button>
        </span>
      </template>
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
      activeTab: 'my-requests',
      loading: false,
      user: null,
      
      // 新建请求表单
      newRequest: {
        mode: 'Fast',
        amount: 30
      },
      
      // 我的请求列表
      myRequests: [],
      
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
  
  created() {
    // 获取用户信息
    const user = JSON.parse(localStorage.getItem('user'))
    if (!user) {
      this.$router.push('/home')
      return
    }
    this.user = user
    
    // 加载数据
    this.loadMyRequests()
    this.loadQueues()
  },
  
  methods: {
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
          this.activeTab = 'my-requests'
          this.loadMyRequests()
          this.loadQueues()
        } else {
          ElMessage.error(response.data.message || '创建失败')
        }
      } catch (error) {
        console.error('创建请求失败:', error)
        ElMessage.error('创建请求失败')
      } finally {
        this.loading = false
      }
    },
    
    // 加载我的请求
    async loadMyRequests() {
      if (!this.user) return
      
      this.loading = true
      try {
        const response = await axios.get(`http://localhost:8080/users/${this.user.id}/charging-requests`)
        if (response.data.success) {
          this.myRequests = response.data.data
        }
      } catch (error) {
        console.error('加载请求失败:', error)
        ElMessage.error('加载请求失败')
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
          this.fastQueue = fastResponse.data.data
        }
        if (slowResponse.data.success) {
          this.slowQueue = slowResponse.data.data
        }
      } catch (error) {
        console.error('加载队列失败:', error)
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
      this.loading = true
      try {
        // 如果模式改变了，需要调用修改模式接口
        if (this.editForm.mode !== this.editForm.originalMode) {
          await axios.put(`http://localhost:8080/charging-requests/${this.editForm.id}/mode`, {
            mode: this.editForm.mode,
            queue_number: '' // 后端会重新生成
          })
        }
        
        // 如果充电量改变了，需要调用修改充电量接口
        const originalRequest = this.myRequests.find(r => r.id === this.editForm.id)
        if (this.editForm.amount !== originalRequest.amount) {
          await axios.put(`http://localhost:8080/charging-requests/${this.editForm.id}/amount`, {
            amount: this.editForm.amount
          })
        }
        
        ElMessage.success('修改成功！')
        this.editDialogVisible = false
        this.loadMyRequests()
        this.loadQueues()
      } catch (error) {
        console.error('修改失败:', error)
        ElMessage.error('修改失败')
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
          this.loadMyRequests()
          this.loadQueues()
        } else {
          ElMessage.error(response.data.message || '取消失败')
        }
      } catch (error) {
        if (error !== 'cancel') {
          console.error('取消请求失败:', error)
          ElMessage.error('取消请求失败')
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
h3 {
  margin-bottom: 15px;
  color: #303133;
}

.el-card {
  border-radius: 8px;
}

.el-tabs {
  background: transparent;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.el-table {
  border-radius: 4px;
  overflow: hidden;
}
</style> 