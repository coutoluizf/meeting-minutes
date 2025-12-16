'use client'

import { useEffect, useState } from 'react'
import { I18nextProvider } from 'react-i18next'
import i18n from '@/i18n/config'
import { invoke } from '@tauri-apps/api/core'

/**
 * I18n Provider that syncs with backend language preference
 *
 * This provider:
 * 1. Fetches language preference from Tauri backend on mount
 * 2. Updates i18n language accordingly
 * 3. Wraps children with I18nextProvider
 *
 * Date: 13/11/2025 - Author: Luiz
 */
export function I18nProvider({ children }: { children: React.ReactNode }) {
  const [isReady, setIsReady] = useState(false)

  useEffect(() => {
    // Fetch language preference from backend
    invoke<string>('api_get_language')
      .then((language) => {
        console.log('🌐 Fetched language from backend:', language)
        i18n.changeLanguage(language)
      })
      .catch((error) => {
        console.error('❌ Failed to fetch language preference:', error)
        // Fallback to default language (pt)
        i18n.changeLanguage('pt')
      })
      .finally(() => {
        setIsReady(true)
      })
  }, [])

  // Show loading indicator while language is being fetched
  // Previously returned null which could cause blank screen if invoke fails
  if (!isReady) {
    return <div className="flex items-center justify-center h-screen">Loading...</div>
  }

  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
}
