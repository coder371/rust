import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    // اسمع على كل الواجهات عشان الجهاز يتفتح من الشبكة بالـ IP
    host: true,
  },
});
