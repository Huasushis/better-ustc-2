import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import ActivityDetailView from '../views/ActivityDetailView.vue'
import ProfileView from '../views/ProfileView.vue'
import RegisteredView from '../views/RegisteredView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/activity/:id', name: 'activity-detail', component: ActivityDetailView, props: true },
    { path: '/profile', name: 'profile', component: ProfileView },
    { path: '/registered', name: 'registered', component: RegisteredView, meta: { hideBack: true } },
    { path: '/about', name: 'about', component: () => import('../views/AboutView.vue') },
    { path: '/logs', name: 'logs', component: () => import('../views/LogView.vue') },
  ],
})

export default router
