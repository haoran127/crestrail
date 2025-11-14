'use client'

import { useState, useEffect } from 'react'
import { usePathname, useRouter } from 'next/navigation'
import DatabaseSelector from './DatabaseSelector'
import SchemaSelector from './SchemaSelector'

interface MenuItem {
  id: string
  name: string
  icon: string
  path?: string
  children?: SubMenuItem[]
}

interface SubMenuItem {
  name: string
  path: string
  badge?: string
  comingSoon?: boolean
}

const menuStructure: MenuItem[] = [
  {
    id: 'home',
    name: '首页',
    icon: 'fa-home',
    path: '/dashboard',
  },
  {
    id: 'database',
    name: '数据库',
    icon: 'fa-database',
    children: [
      { name: 'Schema 可视化', path: '/dashboard/visualizer' },
      { name: '数据表', path: '/dashboard/tables' },
      { name: '表结构设计', path: '/dashboard/table-designer', comingSoon: true },
      { name: '数据导入', path: '/dashboard/import', comingSoon: true },
      { name: '索引管理', path: '/dashboard/indexes', comingSoon: true },
    ],
  },
  {
    id: 'query',
    name: 'SQL 查询',
    icon: 'fa-code',
    children: [
      { name: 'SQL 查询器', path: '/dashboard/query' },
      { name: '查询性能', path: '/dashboard/query-analyzer', comingSoon: true },
      { name: '慢查询日志', path: '/dashboard/slow-queries', comingSoon: true },
      { name: '查询历史', path: '/dashboard/query-history', comingSoon: true },
    ],
  },
  {
    id: 'security',
    name: '角色权限',
    icon: 'fa-shield-alt',
    children: [
      { name: '角色管理', path: '/dashboard/roles', comingSoon: true },
      { name: '权限配置', path: '/dashboard/permissions', comingSoon: true },
      { name: '用户管理', path: '/dashboard/users' },
      { name: '安全审计', path: '/dashboard/audit', comingSoon: true },
    ],
  },
  {
    id: 'monitor',
    name: '监控',
    icon: 'fa-chart-line',
    children: [
      { name: '实时监控', path: '/dashboard/monitor', comingSoon: true },
      { name: '数据库健康', path: '/dashboard/health', comingSoon: true },
      { name: '慢查询', path: '/dashboard/slow-queries', comingSoon: true },
      { name: '数据库连接', path: '/dashboard/connections' },
    ],
  },
  {
    id: 'advanced',
    name: '高级',
    icon: 'fa-cogs',
    children: [
      { name: '函数', path: '/dashboard/functions', comingSoon: true },
      { name: '触发器', path: '/dashboard/triggers', comingSoon: true },
      { name: '事务管理', path: '/dashboard/transaction' },
      { name: '备份恢复', path: '/dashboard/backup', comingSoon: true },
    ],
  },
  {
    id: 'tools',
    name: '工具',
    icon: 'fa-wrench',
    children: [
      { name: 'API 测试', path: '/test' },
      { name: 'Schema 迁移', path: '/dashboard/migrations', comingSoon: true },
      { name: '性能顾问', path: '/dashboard/advisor', comingSoon: true },
    ],
  },
]

export default function SidebarV3() {
  const pathname = usePathname()
  const router = useRouter()
  const [activeGroup, setActiveGroup] = useState<string>('')
  const [showUserMenu, setShowUserMenu] = useState(false)

  // 根据当前路径自动设置激活的分组
  useEffect(() => {
    for (const item of menuStructure) {
      if (item.children) {
        const isInGroup = item.children.some(child => child.path === pathname)
        if (isInGroup) {
          setActiveGroup(item.id)
          break
        }
      }
    }
  }, [pathname])

  const handleLogout = () => {
    localStorage.removeItem('token')
    router.push('/login')
  }

  const isActive = (path: string) => {
    if (path === '/dashboard') {
      return pathname === '/dashboard'
    }
    return pathname === path
  }

  const handleGroupClick = (group: MenuItem) => {
    if (group.path) {
      // 单独的菜单项，直接跳转
      router.push(group.path)
    } else if (group.children && group.children.length > 0) {
      // 有子菜单，跳转到第一个子菜单项
      setActiveGroup(group.id)
      const firstChild = group.children[0]
      if (!firstChild.comingSoon && firstChild.path) {
        router.push(firstChild.path)
      }
    }
  }

  const handleSubMenuClick = (path: string, comingSoon?: boolean) => {
    if (!comingSoon) {
      router.push(path)
    }
  }

  const currentMenu = menuStructure.find(m => m.id === activeGroup)

  return (
    <div className="flex h-screen bg-slate-50">
      {/* 左侧主菜单栏 */}
      <div className="w-[220px] bg-white border-r border-gray-200 flex flex-col">
        {/* Logo */}
        <div className="h-[60px] flex items-center px-5 border-b border-gray-200">
          <div className="flex items-center space-x-2.5">
            <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center shadow-sm">
              <i className="fas fa-database text-white text-sm"></i>
            </div>
            <div>
              <h1 className="text-sm font-semibold text-gray-900">CrestRail</h1>
              <p className="text-[11px] text-gray-500">Database Management</p>
            </div>
          </div>
        </div>

        {/* 数据库选择器 */}
        <div className="px-3 py-3 border-b border-gray-200 space-y-2">
          <DatabaseSelector />
          <SchemaSelector />
        </div>

        {/* 主菜单 */}
        <nav className="flex-1 px-3 py-2 overflow-y-auto">
          <div className="space-y-0.5">
            {menuStructure.map((item) => (
              <button
                key={item.id}
                onClick={() => handleGroupClick(item)}
                className={`w-full flex items-center space-x-3 px-3 py-2 rounded-md text-[13px] transition-all duration-150
                  ${activeGroup === item.id && item.children
                    ? 'bg-gray-100 text-gray-900 font-semibold'
                    : isActive(item.path || '')
                    ? 'bg-blue-50 text-blue-600 font-semibold'
                    : 'text-gray-700 hover:bg-gray-50 hover:text-gray-900'
                  }`}
              >
                <i className={`fas ${item.icon} text-sm w-4 flex-shrink-0 ${
                  activeGroup === item.id && item.children
                    ? 'text-gray-700'
                    : isActive(item.path || '')
                    ? 'text-blue-600'
                    : 'text-gray-400'
                }`}></i>
                <span className="flex-1 text-left">{item.name}</span>
                {item.children && activeGroup === item.id && (
                  <i className="fas fa-chevron-right text-[10px] text-gray-400"></i>
                )}
              </button>
            ))}
          </div>
        </nav>

        {/* 底部用户信息 */}
        <div className="border-t border-gray-200 p-3 relative">
          <button
            onClick={() => setShowUserMenu(!showUserMenu)}
            className="w-full flex items-center space-x-2.5 px-3 py-2 rounded-lg hover:bg-gray-50 transition-colors"
          >
            <div className="w-7 h-7 bg-gradient-to-br from-blue-500 to-blue-600 rounded-full flex items-center justify-center flex-shrink-0">
              <span className="text-white text-xs font-medium">A</span>
            </div>
            <div className="flex-1 min-w-0 text-left">
              <p className="text-xs font-medium text-gray-900 truncate">admin</p>
              <p className="text-[11px] text-gray-500 truncate">管理员</p>
            </div>
            <i className={`fas fa-chevron-down text-gray-400 text-[10px] flex-shrink-0 transition-transform ${showUserMenu ? 'rotate-180' : ''}`}></i>
          </button>
          
          {/* 用户下拉菜单 */}
          {showUserMenu && (
            <div className="absolute bottom-full left-3 right-3 mb-2 bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden">
              <div className="py-1">
                <button
                  className="w-full flex items-center space-x-2.5 px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                >
                  <i className="fas fa-user text-xs w-4 text-gray-400"></i>
                  <span>个人资料</span>
                </button>
                <button
                  className="w-full flex items-center space-x-2.5 px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                >
                  <i className="fas fa-cog text-xs w-4 text-gray-400"></i>
                  <span>设置</span>
                </button>
                <div className="border-t border-gray-100 my-1"></div>
                <button
                  onClick={handleLogout}
                  className="w-full flex items-center space-x-2.5 px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
                >
                  <i className="fas fa-sign-out-alt text-xs w-4"></i>
                  <span>退出登录</span>
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* 右侧子菜单栏 */}
      {currentMenu?.children && (
        <div className="w-[200px] bg-white border-r border-gray-200 flex flex-col">
          {/* 子菜单标题 */}
          <div className="h-[60px] flex items-center px-4 border-b border-gray-200">
            <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-wider">
              {currentMenu.name}
            </h2>
          </div>

          {/* 子菜单列表 */}
          <nav className="flex-1 px-3 py-3 overflow-y-auto">
            <div className="space-y-0.5">
              {currentMenu.children.map((subItem) => (
                <button
                  key={subItem.path}
                  onClick={() => handleSubMenuClick(subItem.path, subItem.comingSoon)}
                  disabled={subItem.comingSoon}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-md text-[13px] transition-all duration-150
                    ${isActive(subItem.path)
                      ? 'bg-blue-50 text-blue-600 font-semibold'
                      : subItem.comingSoon
                      ? 'text-gray-400 cursor-not-allowed'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-gray-900'
                    }`}
                >
                  <span>{subItem.name}</span>
                  {subItem.comingSoon && (
                    <span className="text-[10px] px-1.5 py-0.5 bg-gray-100 text-gray-500 rounded font-medium">
                      Soon
                    </span>
                  )}
                  {subItem.badge && (
                    <span className="text-[10px] px-1.5 py-0.5 bg-blue-100 text-blue-600 rounded font-medium">
                      {subItem.badge}
                    </span>
                  )}
                </button>
              ))}
            </div>
          </nav>

          {/* 帮助链接 */}
          <div className="border-t border-gray-200 px-3 py-3">
            <a
              href="#"
              className="flex items-center space-x-2 px-3 py-2 text-xs text-gray-600 hover:text-gray-900 transition-colors"
            >
              <i className="fas fa-question-circle"></i>
              <span>帮助文档</span>
            </a>
          </div>
        </div>
      )}
    </div>
  )
}

