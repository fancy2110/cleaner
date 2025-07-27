import { createApp } from 'vue';
import App from './App.vue';
import i18n from './i18n';
import PrimeVue from 'primevue/config';
import Aura from '@primeuix/themes/aura';
import ConfirmationService from 'primevue/confirmationservice';
import ToastService from 'primevue/toastservice';

import '@/assets/main.css';
import '@/assets/styles.scss';
import router from '@/router/index';

const app = createApp(App);
app.use(router);
app.use(PrimeVue, {
    theme: {
        preset: Aura,
        options: {
            darkModeSelector: '.app-dark'
        }
    }
});
app.use(ToastService);
app.use(ConfirmationService);

// Add i18n to the app
app.use(i18n);

app.mount('#app');
