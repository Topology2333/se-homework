<template>
  <el-container style="background: #f5f7fa;">
    <!-- 只在非登录页显示侧边栏 -->
    <el-aside v-if="showSidebar" width="200px"
      style="background: #fff; display: flex; flex-direction: column; justify-content: space-between;">
      <!-- 顶部 Logo -->
      <div style="padding: 10px; text-align: center;">
        <img src="./assets/1.jpg" alt="Logo" style="max-width: 100%; height: auto;" />
      </div>

      <!-- 菜单 -->
      <el-menu :default-active="activeMenu" @select="handleMenuSelect" class="el-menu-vertical-demo"
        active-text-color="#409EFF" background-color="#FFF4E2" text-color="#000"
        style="flex-grow: 1; display: flex; flex-direction: column; justify-content: center; align-items: center; padding: 0;">
        <el-menu-item index="main">主页</el-menu-item>
        <el-menu-item index="request">用户请求</el-menu-item>
        <el-menu-item v-if="isAdmin" index="admin">管理员</el-menu-item>

        <!-- 底部退出按钮 -->
        <div style="margin-top: 20vh;padding: 20px; text-align: center; border-top: 1px solid #eee;">
          <el-button type="danger" @click="logout" style="width: 100%;">退出</el-button>
        </div>
      </el-menu>


    </el-aside>

    <!-- 主体内容区域 -->
    <el-main class="app-bg" style="overflow: auto;">
      <router-view />
    </el-main>
  </el-container>
</template>

<script>
export default {
  data() {
    return {
      activeMenu: this.$route.path, // 初始设置激活菜单项
      isAdmin: false
    };
  },
  methods: {
    // 菜单项点击时的回调，更新路由
    handleMenuSelect(index) {
      this.$router.push({ path: `/${index}` });
    },
    logout() {
      this.isAdmin = false;     //防止客户在刷新页面访问到admin
      this.$router.push('/home');
    },
    checkAdminStatus() {
      const user = JSON.parse(localStorage.getItem('user'));
      if (user && user.is_admin) {
        this.isAdmin = true; // 如果是管理员，更新 isAdmin 状态
      } else {
        this.isAdmin = false; // 否则设置为 false
      }
    }
  },
  computed: {
    showSidebar() {
      // 登录页不显示侧边栏，假设登录页路径为 "/login"
      return this.$route.path !== '/home' && this.$route.path !== '/register'
    },
  },
  watch: {
    '$route.path'(newPath) {
      // 更新菜单的选中状态
      this.activeMenu = newPath.replace('/', '');
    }
  },
  mounted() {
    // 页面加载时就通过 localStorage 判断是否为管理员
    this.checkAdminStatus();
  },
};
</script>

<style>
.app-bg {
  height: 98vh;
  background-image: url('./assets/2.jpg');
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

.el-menu-vertical-demo {
  width: 100%;
  height: 100%;
  padding: 0;
}

.el-menu-item {
  width: 100%;
  text-align: center;
  /* 水平居中文本 */
}
</style>