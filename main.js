async function main(){
    let greeting = document.getElementById("greeting");
    let appKey = localStorage.getItem("appKey");
    let buton = document.getElementById("login-wv-button");
    buton.setAttribute("data-callback","/callback")
    if(appKey == null){
      let newscript = document.createElement("script");
      newscript.src = "https://mathwow.org/dash/wv/vortice.js";
      document.head.appendChild(newscript);
    } else {
      let response = await fetch("https://mathwow.org/api/apps/" + appKey + "/userdata");
      response = await response.json();
      greeting.innerHTML = "Hello, " + response.name;
    }
    document.addEventListener("message", function(event){
      if(event.data.error){
        console.error("error logging in with vortice: " + event.data.error);
      } else {
        localStorage.setItem("appKey", event.data);
        window.location.reload();
      }
    });
  }
  document.addEventListener("DOMContentLoaded", () => main());
