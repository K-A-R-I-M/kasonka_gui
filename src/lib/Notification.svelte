<script lang="ts">
  export let audio_title: string;
</script>

<div
  class="bg-teal-100 border-t-4 border-teal-500 rounded-b text-teal-900 px-4 py-3 shadow-md"
  role="alert"
>
  <div class="flex flex-col">
    <p class="font-bold">Start downloading</p>
    <p class="text-sm">"{audio_title}" will be downloaded</p>
  </div>
</div>
