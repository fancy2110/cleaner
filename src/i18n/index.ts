import { createI18n } from 'vue-i18n';
import en from '../locales/en.json';
import zh from '../locales/zh.json';
import es from '../locales/es.json';
import ja from '../locales/ja.json';

// Define supported languages
export const availableLanguages = [
  { code: 'en', name: 'English' },
  { code: 'zh', name: '中文' },
  { code: 'es', name: 'Español' },
  { code: 'ja', name: '日本語' }
];

// Get preferred language from system
const getBrowserLanguage = (): string => {
  try {
    const navigatorLanguage = navigator.language.split('-')[0];
    
    // Check if the browser language is supported
    if (['en', 'zh', 'es', 'ja'].includes(navigatorLanguage)) {
      return navigatorLanguage;
    }
  } catch (e) {
    console.error('Error detecting browser language:', e);
  }
  
  return 'en'; // Default to English
};

// Try to get language from localStorage
const getSavedLanguage = (): string => {
  try {
    const savedLanguage = localStorage.getItem('app-language');
    if (savedLanguage && ['en', 'zh', 'es', 'ja'].includes(savedLanguage)) {
      return savedLanguage;
    }
  } catch (error) {
    console.error('Error accessing localStorage:', error);
  }
  
  return getBrowserLanguage();
};

// Create i18n instance
const i18n = createI18n({
  legacy: false,
  locale: getSavedLanguage(),
  fallbackLocale: 'en',
  messages: {
    en,
    zh,
    es,
    ja
  }
});

// Save language preference
export const setLanguage = (lang: string): void => {
  if (['en', 'zh', 'es', 'ja'].includes(lang)) {
    try {
      localStorage.setItem('app-language', lang);
      // @ts-ignore - We know the i18n instance has global
      i18n.global.locale.value = lang;
    } catch (error) {
      console.error('Error saving language preference:', error);
    }
  }
};

export default i18n;