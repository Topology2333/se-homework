import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import Register from '../views/Register.vue'
import Main from '@/views/Main.vue'
import Admin from '@/views/Admin.vue'
import Details from '@/views/Details.vue'
import Request from '@/views/Request.vue'

const routes = [
  { path: '/', redirect: '/home' },
  { path: '/home', component: Home },
  { path: '/register', component: Register },
  { path: '/main', component: Main },
  { path: '/admin', component: Admin },
  { path: '/details', component: Details },
  { path: '/request', component: Request }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router