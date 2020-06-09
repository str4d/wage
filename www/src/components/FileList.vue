<template>
  <ul id="file-list" class="fa-ul">
    <li v-for="(file, index) in files" :key="file.name">
      <span class="fa-li">
        <font-awesome-icon :icon="getClassNameForMimeType(file.type).substring(3)" />
      </span>
      {{ file.name }} ({{ file.size }} b)
      <button @click="removeFile(index)" title="Remove">X</button>
    </li>
  </ul>
</template>

<script>
import { getClassNameForMimeType } from "font-awesome-filetypes";

export default {
  name: "FileList",
  props: ["files"],
  methods: {
    getClassNameForMimeType,
    removeFile(index) {
      this.$emit("file-removed", index);
    }
  }
};
</script>

<style scoped>
#file-list {
  font-family: sans-serif;
  padding: 20px;
}
</style>
