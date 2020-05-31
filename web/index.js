window.onload = function() {
    console.log("ready!")
    let zip = null;
    document.querySelector("#submit").addEventListener("click", (e) => {
        let params = {
            light: document.querySelector("#light").value
        };
        document.querySelector("#display").innerHTML = "";
        let files = document.querySelector("#files").files;
        zip = new JSZip();
        let target_size = getSize(files.length);
        for(let i = 0; i < files.length; i++) {
            let file = files[i];
            render_api(file, params)
                .then((r) => {
                    let src = window.URL.createObjectURL(r.blob);
                    let img = document.createElement("img");
                    img.src = src;
                    img.width = target_size;
                    img.height = target_size;
                    document.querySelector("#display").appendChild(img);
                    let id = r.id;
                    zip.file(id.x + ',' + id.z + ".png", r.blob);
                })
                .catch((r) => {
                    let id = r.id;
                    if(id instanceof Object) {
                        id = "tile(" + id.x + "," + z + ")";
                    }
                    let img = document.createElement("img");
                    img.src = "#";
                    img.width = target_size;
                    img.height = target_size;
                    img.alt = id + ' ' + r.status;
                    document.querySelector("#display").appendChild(img);
                })
            
            
        }
    })
    document.querySelector("#save").addEventListener("click", (ev) => {
        if(zip !== null) {
            let div = document.createElement("div");
            div.innerText = "calculating...";
            document.querySelector("#saving").appendChild(div);
            zip.generateAsync({type: "blob"})
                .then((content) => {
                    div.remove();
                    let link = document.createElement("a");
                    link.href = window.URL.createObjectURL(content);
                    link.innerText = "DOWNLOAD"
                    link.download = "test.zip"
                    link.addEventListener("click", (ev) => {
                        link.remove();
                    })
                    document.querySelector("#saving").appendChild(link);
                })
                .catch((e) => {
                    div.remove();
                    console.log(e);
                })
        }
    })
}

function getSize(n) {
    let tw = window.innerWidth;
    let th = window.innerHeight;
    let sz = 256;
    for(; sz > 64; sz = sz / 2) {
        let lw = Math.floor(tw / sz);
        let lh = Math.floor(th / sz);
        if(lw * lh >= n) {
            break;
        }
    }
    return sz;
}
// function upload(file) {
//     let form = new FormData();
//     form.append("tile", file);
//     let req = new XMLHttpRequest();
//     req.addEventListener("load", function(ev) {
//         let blob = this.response;
//         let src = window.URL.createObjectURL(blob)
//         // let img = document.createElement("img");
//         // img.src = src;
//         // img.onload = function() {
//         //     document.querySelector("#display").appendChild(img)
//         // }
//         let a = document.createElement("a");
//         a.innerText = file.name.replace(".zip", ".png");
//         a.href = src;
//         a.download = file.name.replace(".zip", ".png");
//         let d = document.createElement("div");
//         d.appendChild(a);
//         document.querySelector("#display").appendChild(d);
//         window.setTimeout(function() {
//             a.click();
//         }, 10)
//     });
//     req.responseType = "blob"
//     req.open("POST", "/render")
//     req.send(form)
// }


/**
 * 
 * @param {File} file 
 * @param {Object} params
 */
function render_api(file, params) {
    const pattern = /(-?\d+),(-?\d+)\.zip/;
    let data = new FormData()
    data.append("tile", file)
    let url = "/render";
    if(params instanceof Object) {
        let query = new Array();
        for(let key in params) {
            let value = params[key]
            query.push(key + "=" + value)
        }
        if(query.length > 0) {
            url += ('?' + query.join('&'))
        }
    }
    let id = pattern.exec(file.name)
    if(id !== null) {
        id = {
            x: id[1],
            z: id[2]
        };
    } else {
        id = file.name;
    }
    return new Promise(function (resolve, reject) {
        let req = new XMLHttpRequest();
        req.onload = function(ev) {
            if(this.status == 200) {
                let blob = this.response
                resolve({
                    id,
                    blob
                })
            } else {
                reject({
                    id,
                    status: this.status,
                    message: this.responseText,
                })
            }
        }
        req.onerror = function(ev) {
            reject({
                id,
                status: this.status,
                message: this.responseText,
            })
        }
        req.responseType = "blob";
        req.open("POST", url);
        req.send(data)
    });
}