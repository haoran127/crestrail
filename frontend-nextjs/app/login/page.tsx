'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { useAppStore } from '@/lib/store'
import axios from 'axios'

export default function LoginPage() {
  const router = useRouter()
  const setCurrentUser = useAppStore(state => state.setCurrentUser)
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError('')

    try {
      const response = await axios.post('/auth/login', {
        email,
        password,
      })

      const { token, user } = response.data
      
      // 保存 token 和用户信息
      localStorage.setItem('token', token)
      setCurrentUser(user)
      
      console.log('✅ 登录成功:', user)
      
      router.push('/dashboard')
    } catch (err: any) {
      setError(err.response?.data?.error || '登录失败')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500">
      {/* 左侧品牌展示 */}
      <div className="flex-1 flex items-center justify-center p-8 lg:p-16">
        <div className="text-white max-w-lg space-y-8">
          <div className="space-y-6">
            <div className="flex items-center space-x-4 group">
              <div className="w-14 h-14 bg-white/10 backdrop-blur-lg rounded-xl flex items-center justify-center 
                            shadow-2xl transform transition-all duration-300 group-hover:scale-110 group-hover:rotate-3
                            border border-white/20">
                <i className="fas fa-database text-3xl text-white"></i>
              </div>
              <div>
                <h1 className="text-4xl font-bold tracking-tight">CrestRail</h1>
                <p className="text-sm opacity-90 font-light">Enterprise Database Management</p>
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <h2 className="text-3xl font-semibold leading-tight">
              PostgreSQL<br />统一管理平台
            </h2>
            <p className="text-lg opacity-90 leading-relaxed font-light">
              提供完整的数据库管理、查询分析、性能监控和运维工具，帮助企业高效管理 PostgreSQL 数据库。
            </p>
          </div>

          <div className="grid grid-cols-2 gap-4">
            {['Schema 管理', 'SQL 查询', '性能监控', '数据导出'].map((feature, idx) => (
              <div
                key={idx}
                className="flex items-center space-x-3 bg-white/10 backdrop-blur-sm rounded-lg p-3 
                          transition-all duration-300 hover:bg-white/20 hover:scale-105 cursor-default"
              >
                <div className="w-8 h-8 bg-white/20 rounded-lg flex items-center justify-center">
                  <i className={`fas fa-${['database', 'code', 'chart-line', 'download'][idx]} text-sm`}></i>
                </div>
                <span className="text-sm font-medium">{feature}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* 右侧登录表单 */}
      <div className="w-full lg:w-[480px] bg-white flex items-center justify-center p-8 shadow-2xl">
        <div className="w-full max-w-sm space-y-8">
          <div className="space-y-2">
            <h3 className="text-2xl font-semibold text-gray-800">登录账户</h3>
            <p className="text-sm text-gray-500">欢迎回来，请登录您的账户</p>
          </div>

          <form onSubmit={handleLogin} className="space-y-5">
            <div className="space-y-2">
              <label className="block text-sm font-medium text-gray-700">邮箱地址</label>
              <div className="relative">
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  autoComplete="email"
                  className="w-full input-with-icon pl-10"
                  placeholder="请输入邮箱地址"
                />
                <i className="fas fa-envelope absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 text-sm pointer-events-none"></i>
              </div>
            </div>

            <div className="space-y-2">
              <label className="block text-sm font-medium text-gray-700">密码</label>
              <div className="relative">
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  autoComplete="current-password"
                  className="w-full input-with-icon pl-10"
                  placeholder="请输入密码"
                />
                <i className="fas fa-lock absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 text-sm pointer-events-none"></i>
              </div>
            </div>

            {error && (
              <div className="flex items-start space-x-2 text-sm text-red-600 bg-red-50 border border-red-200 rounded p-3">
                <i className="fas fa-exclamation-circle mt-0.5"></i>
                <span>{error}</span>
              </div>
            )}

            <button
              type="submit"
              disabled={loading}
              className="w-full h-10 bg-primary-500 hover:bg-primary-400 active:bg-primary-600 
                       text-white font-medium rounded-lg shadow-lg hover:shadow-xl
                       transform transition-all duration-200 hover:-translate-y-0.5
                       focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2
                       disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
            >
              {loading ? (
                <span className="flex items-center justify-center space-x-2">
                  <i className="fas fa-spinner fa-spin"></i>
                  <span>登录中...</span>
                </span>
              ) : (
                <span className="flex items-center justify-center space-x-2">
                  <span>登录</span>
                  <i className="fas fa-arrow-right text-sm"></i>
                </span>
              )}
            </button>
          </form>

          <div className="pt-6 border-t border-gray-100">
            <div className="bg-blue-50 border border-blue-100 rounded-lg p-4">
              <p className="text-xs font-medium text-blue-800 mb-2">
                <i className="fas fa-info-circle mr-1"></i>
                测试账号
              </p>
              <div className="text-xs text-blue-700 space-y-1 font-mono">
                <p>邮箱：admin@example.com</p>
                <p>密码：Admin123</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

