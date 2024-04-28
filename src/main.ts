import { invoke } from '@tauri-apps/api'

let save_button = document.getElementById("button") as HTMLButtonElement;

save_button.addEventListener("click", function () {
  let username = document.getElementById("identifier") as HTMLInputElement
  let password = document.getElementById("password") as HTMLInputElement
  if (username.value && password.value) {
    invoke("save_credentials", { username: username.value, password: password.value }).then(r  => console.log(r));
  } else {
    console.log("a")
  }
})