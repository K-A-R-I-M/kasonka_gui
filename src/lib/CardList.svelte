<script lang="ts">
  import Card from "$lib/Card.svelte";
  import type { Item } from "$lib/types";

  export let listItems: Array<Item>;
</script>

<div class="list">
  {#each listItems as item}
    <Card title={item.title} duration={item.duration} />
  {/each}
</div>
