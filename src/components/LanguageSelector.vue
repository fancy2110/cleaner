<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { availableLanguages, setLanguage } from '../i18n';
import { useI18n } from 'vue-i18n';

const { locale } = useI18n();
const isOpen = ref(false);
const selectedLanguage = ref(locale.value);

// Set language and close dropdown
const changeLanguage = (langCode: string) => {
  setLanguage(langCode);
  selectedLanguage.value = langCode;
  isOpen.value = false;
};

// Toggle dropdown
const toggleDropdown = () => {
  isOpen.value = !isOpen.value;
};

// Close dropdown when clicking outside
const closeDropdown = (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (!target.closest('.language-selector')) {
    isOpen.value = false;
  }
};

// Get language display name
const getLanguageName = (code: string): string => {
  const lang = availableLanguages.find(lang => lang.code === code);
  return lang ? lang.name : code;
};

// Get language flag emoji
const getLanguageFlag = (code: string): string => {
  switch (code) {
    case 'en': return '🇺🇸';
    case 'zh': return '🇨🇳';
    case 'es': return '🇪🇸';
    case 'ja': return '🇯🇵';
    default: return '🌐';
  }
};

// Add event listener for clicking outside
onMounted(() => {
  document.addEventListener('click', closeDropdown);
  
  // Cleanup function to remove event listener
  return () => {
    document.removeEventListener('click', closeDropdown);
  };
});
</script>

<template>
  <div class="language-selector">
    <button 
      class="language-button" 
      @click.stop="toggleDropdown"
      :title="getLanguageName(selectedLanguage)"
    >
      <span class="language-flag">{{ getLanguageFlag(selectedLanguage) }}</span>
      <span class="language-code">{{ selectedLanguage.toUpperCase() }}</span>
      <span class="dropdown-arrow" :class="{ 'open': isOpen }">▾</span>
    </button>
    
    <div class="language-dropdown" v-if="isOpen">
      <button
        v-for="language in availableLanguages"
        :key="language.code"
        class="language-option"
        :class="{ 'active': language.code === selectedLanguage }"
        @click="changeLanguage(language.code)"
      >
        <span class="language-flag">{{ getLanguageFlag(language.code) }}</span>
        <span class="language-name">{{ language.name }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.language-selector {
  position: relative;
  z-index: 100;
}

.language-button {
  display: flex;
  align-items: center;
  gap: 6px;
  background-color: transparent;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 6px 10px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.language-button:hover {
  background-color: #f5f5f5;
}

.language-flag {
  font-size: 16px;
}

.language-code {
  font-size: 14px;
  font-weight: 500;
}

.dropdown-arrow {
  font-size: 12px;
  transition: transform 0.2s;
}

.dropdown-arrow.open {
  transform: rotate(180deg);
}

.language-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  width: 150px;
  overflow: hidden;
}

.language-option {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  text-align: left;
  padding: 8px 12px;
  border: none;
  background-color: transparent;
  cursor: pointer;
}

.language-option:hover {
  background-color: #f5f5f5;
}

.language-option.active {
  background-color: #e6f7ff;
  font-weight: 500;
}

.language-name {
  flex: 1;
}

@media (max-width: 768px) {
  .language-code {
    display: none;
  }
  
  .language-button {
    padding: 6px;
  }
}
</style>