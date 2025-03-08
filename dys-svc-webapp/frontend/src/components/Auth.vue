<script setup lang="ts">
import {computed, inject, onMounted, ref} from "vue";
import type {VueCookies} from "vue-cookies";
import { CreateAccountRequest } from "%/services/auth/account.ts";
const $cookies = inject<VueCookies>('$cookies');
const cookie = ref($cookies?.get("dax-auth"));
const isLoggedIn = computed(() => cookie.value && (cookie.value.length > 0));

const createAccount = async () => {
  const accountName = (document.getElementById("auth-username") as HTMLInputElement).value;
  const request = CreateAccountRequest.fromJSON({
    accountName: accountName
  });

  const respStatus = (await fetch(`api/create_account`, {
    method: "POST",
    body: JSON.stringify(request)
  })).status;
  if (respStatus === 200) {
    $cookies?.set("dax-auth", accountName);
    cookie.value = accountName;
  }
};

const logout = async () => {
  $cookies?.remove('dax-auth');
  cookie.value = null;
};

onMounted(() => {
  cookie.value = $cookies?.get("dax-auth");
})
</script>

<template>
  <div v-if="isLoggedIn">
    <p>Authenticated as {{cookie}}</p>
    <p @click="logout">Log Out</p>
  </div>
  <div v-else>
    <label for="username">Username (TEMP)</label>
    <input id="auth-username" name="username" type="text"/>
    <br>
    <button @click="createAccount">Create Account</button>
  </div>
</template>

<style scoped>
div {
  margin-left: auto;
}
</style>