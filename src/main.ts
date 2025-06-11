import { createApp } from 'vue';
import App from './App.vue';
import i18n from './i18n';
import './assets/main.css';

const app = createApp(App);

// Add i18n to the app
app.use(i18n);

app.mount('#app');