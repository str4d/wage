<template>
  <section class="columns">
    <div class="column is-narrow">
      <font-awesome-icon :icon="fileIcon" size="10x" />
    </div>
    <div id="file-info" class="column">
      <p>TODO: file info.</p>
    </div>
  </section>
</template>

<script>
export default {
  name: "FileInfo",
  props: ["fileIcon"],
};
</script>

<style scoped>
#file-info {
  text-align: left;
}
</style>
