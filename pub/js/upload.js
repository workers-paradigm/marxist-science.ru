async function processUpload(e) {
    if (e.preventDefault) e.preventDefault();

    let formData = new FormData(e.target);
    e.target.reset();

    fetch("/api.upload", {
        method: "POST",
        body: formData,
    }).then(response => response.json())
        .then(json => {
            let errorSpan = document.getElementById("upload-error");
            if (json.error.length !== 0) {
                let error = json["error"];
                errorSpan.style = "display:initial;";
                errorSpan.innerHTML = error;
                return;
            }

            errorSpan.style = "display: none;";
            let uploadList = document.getElementById("upload-files");
            let li = document.createElement("li");
            let input = document.createElement("input");
            input.setAttribute("readonly", "");
            input.setAttribute("value", json.contentUrl);
            let image = document.createElement("img");
            image.setAttribute("src", json.contentUrl);
            image.setAttribute("class", "uploaded-image");
            li.append(image);
            li.append(input);
            uploadList.prepend(li);
        });

    return false;
}
let form = document.getElementById("upload-form");
form.addEventListener("submit", processUpload);
