import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://neverwe1come.github.io',
  base: '/KMIPKit',
  integrations: [
    starlight({
      title: 'KMIPKit',
      description: 'Enterprise OASIS KMIP 2.1 Multilanguage Client Library & Infrastructure',
      defaultLocale: 'es',
      locales: {
        es: {
          label: 'Español',
          lang: 'es',
        },
        en: {
          label: 'English',
          lang: 'en',
        },
      },
      social: {
        github: 'https://github.com/NeverWe1come/KMIPKit',
      },
      sidebar: [
        {
          label: '1. Inicio Rápido',
          translations: { en: '1. Getting Started' },
          autogenerate: { directory: 'getting-started' },
        },
        {
          label: '2. Despliegue',
          translations: { en: '2. Deployment & Installation' },
          autogenerate: { directory: 'deployment' },
        },
        {
          label: '3. Pruebas y Test Server',
          translations: { en: '3. Testing & Test Server' },
          autogenerate: { directory: 'testing-and-harness' },
        },
        {
          label: '4. Protocolo KMIP & TTLV',
          translations: { en: '4. KMIP Protocol & TTLV' },
          autogenerate: { directory: 'kmip-protocol' },
        },
        {
          label: '5. Diccionario de Operaciones',
          translations: { en: '5. Operations Dictionary' },
          autogenerate: { directory: 'operations' },
        },
        {
          label: '6. Recetas de Producción',
          translations: { en: '6. Production Cookbook' },
          autogenerate: { directory: 'cookbook' },
        },
        {
          label: '7. Guías de HSMs y KMS',
          translations: { en: '7. HSM & KMS Guides' },
          autogenerate: { directory: 'hsm-guides' },
        },
        {
          label: '8. Arquitectura Interna',
          translations: { en: '8. Workspace Internals' },
          autogenerate: { directory: 'internals' },
        },
        {
          label: '9. Seguridad y Amenazas',
          translations: { en: '9. Security & Threat Model' },
          autogenerate: { directory: 'security' },
        },
        {
          label: '10. Conformidad y Gobernanza',
          translations: { en: '10. Compliance & Governance' },
          autogenerate: { directory: 'compliance' },
        },
      ],
    }),
  ],
});
