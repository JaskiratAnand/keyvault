<script lang="ts">
import { Key } from 'lucide-svelte';

const handleClick = () => {
  window.parent.postMessage({ type: 'keyvault-badge-clicked' }, '*');
};
</script>

<button
  type="button"
  onclick={handleClick}
  class="flex items-center justify-center w-5 h-5 text-zinc-500 hover:text-accent transition-colors duration-150 cursor-pointer focus:outline-none"
  title="KeyVault Autofill"
>
  <Key class="h-4 w-4" />
</button>

<style>
:global(html, body) {
  margin: 0 !important;
  padding: 0 !important;
  width: 20px !important;
  height: 20px !important;
  overflow: hidden !important;
  background-color: transparent !important;
}
button {
  background: transparent;
  border: none;
  padding: 0;
  width: 20px;
  height: 20px;
}
</style>
