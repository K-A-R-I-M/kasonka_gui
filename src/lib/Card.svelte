<script lang="ts">
  export let title: string;
  export let duration: string;
</script>

<div class="bg-slate-900 text-white p-2 rounded-md card mb-3">
  <p class="text-base text-center font-bold">{title}</p>
  <p class="text-end px-4 pt-2">{duration}</p>
</div>
