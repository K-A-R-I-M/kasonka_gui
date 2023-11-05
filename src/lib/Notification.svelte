<script lang="ts">
  export let title: string;
  export let text: string;
</script>

<div
  class="bg-teal-100 border-t-4 border-teal-500 rounded-b text-teal-900 px-4 py-3 shadow-md"
  role="alert"
>
  <div class="flex flex-col">
    <p class="font-bold">{title}</p>
    <p class="text-sm">{text}</p>
  </div>
</div>
