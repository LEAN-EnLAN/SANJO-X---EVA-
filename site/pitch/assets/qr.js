/* Tiny self-contained QR encoder (byte mode, versions 1–10, ECC L/M/Q/H, automatic mask).
   Follows the structure of Project Nayuki's QR Code generator (MIT). Returns a boolean matrix [y][x]. */
(function(g){
const ECC_CW={L:[-1,7,10,15,20,26,18,20,24,30,18],M:[-1,10,16,26,18,24,16,18,22,22,26],Q:[-1,13,22,18,26,18,24,18,22,20,24],H:[-1,17,28,22,16,22,28,26,26,24,28]};
const ECC_BLK={L:[-1,1,1,1,1,1,2,2,2,2,4],M:[-1,1,1,1,2,2,4,4,4,5,5],Q:[-1,1,1,2,2,4,4,6,6,8,8],H:[-1,1,1,2,4,4,4,5,6,8,8]};
const FMT={L:1,M:0,Q:3,H:2};
const bit=(x,i)=>((x>>>i)&1)!==0;
function rawModules(v){let r=(16*v+128)*v+64;if(v>=2){const n=Math.floor(v/7)+2;r-=(25*n-10)*n-55;if(v>=7)r-=36}return r}
function dataCW(v,e){return Math.floor(rawModules(v)/8)-ECC_CW[e][v]*ECC_BLK[e][v]}
function mul(x,y){let z=0;for(let i=7;i>=0;i--){z=(z<<1)^((z>>>7)*0x11D);z^=((y>>>i)&1)*x}return z&255}
function divisor(d){const r=new Array(d).fill(0);r[d-1]=1;let root=1;for(let i=0;i<d;i++){for(let j=0;j<r.length;j++){r[j]=mul(r[j],root);if(j+1<r.length)r[j]^=r[j+1]}root=mul(root,2)}return r}
function remainder(data,div){const r=div.map(()=>0);for(const b of data){const f=b^r.shift();r.push(0);div.forEach((c,i)=>r[i]^=mul(c,f))}return r}
function alignPos(v,size){if(v===1)return[];const n=Math.floor(v/7)+2;const step=Math.ceil((v*4+4)/(n*2-2))*2;const r=[6];for(let p=size-7;r.length<n;p-=step)r.splice(1,0,p);return r}
function encode(text,ecl,forceMask){ecl=ecl||'M';
  const bytes=Array.from(new TextEncoder().encode(text));
  let v=1;for(;v<=10;v++){const cc=v<10?8:16;if(4+cc+bytes.length*8<=dataCW(v,ecl)*8)break}
  if(v>10)throw new Error('QR: text too long');
  const cap=dataCW(v,ecl)*8,bb=[],push=(val,n)=>{for(let i=n-1;i>=0;i--)bb.push((val>>>i)&1)};
  push(4,4);push(bytes.length,v<10?8:16);bytes.forEach(b=>push(b,8));
  push(0,Math.min(4,cap-bb.length));push(0,(8-bb.length%8)%8);
  for(let p=0xEC;bb.length<cap;p^=0xEC^0x11)push(p,8);
  const data=[];for(let i=0;i<bb.length;i+=8){let b=0;for(let j=0;j<8;j++)b=(b<<1)|bb[i+j];data.push(b)}
  // ECC + interleave
  const nb=ECC_BLK[ecl][v],el=ECC_CW[ecl][v],raw=Math.floor(rawModules(v)/8),nShort=nb-raw%nb,shortLen=Math.floor(raw/nb),div=divisor(el),blocks=[];
  for(let i=0,k=0;i<nb;i++){const dat=data.slice(k,k+shortLen-el+(i<nShort?0:1));k+=dat.length;const ecc=remainder(dat,div);if(i<nShort)dat.push(0);blocks.push(dat.concat(ecc))}
  const cw=[];for(let i=0;i<blocks[0].length;i++)blocks.forEach((b,j)=>{if(i!==shortLen-el||j>=nShort)cw.push(b[i])});
  // matrix
  const size=v*4+17,M=[],F=[];for(let y=0;y<size;y++){M.push(new Array(size).fill(false));F.push(new Array(size).fill(false))}
  const set=(x,y,d)=>{M[y][x]=d;F[y][x]=true};
  for(let i=0;i<size;i++){set(6,i,i%2===0);set(i,6,i%2===0)}
  const finder=(cx,cy)=>{for(let dy=-4;dy<=4;dy++)for(let dx=-4;dx<=4;dx++){const d=Math.max(Math.abs(dx),Math.abs(dy)),x=cx+dx,y=cy+dy;if(x>=0&&x<size&&y>=0&&y<size)set(x,y,d!==2&&d!==4)}};
  finder(3,3);finder(size-4,3);finder(3,size-4);
  const ap=alignPos(v,size),na=ap.length;
  for(let i=0;i<na;i++)for(let j=0;j<na;j++){if((i===0&&j===0)||(i===0&&j===na-1)||(i===na-1&&j===0))continue;
    for(let dy=-2;dy<=2;dy++)for(let dx=-2;dx<=2;dx++)set(ap[i]+dx,ap[j]+dy,Math.max(Math.abs(dx),Math.abs(dy))!==1)}
  const drawFormat=mask=>{const d=FMT[ecl]<<3|mask;let r=d;for(let i=0;i<10;i++)r=(r<<1)^((r>>>9)*0x537);const b=(d<<10|r)^0x5412;
    for(let i=0;i<=5;i++)set(8,i,bit(b,i));set(8,7,bit(b,6));set(8,8,bit(b,7));set(7,8,bit(b,8));
    for(let i=9;i<15;i++)set(14-i,8,bit(b,i));
    for(let i=0;i<8;i++)set(size-1-i,8,bit(b,i));for(let i=8;i<15;i++)set(8,size-15+i,bit(b,i));set(8,size-8,true)};
  drawFormat(0);
  if(v>=7){let r=v;for(let i=0;i<12;i++)r=(r<<1)^((r>>>11)*0x1F25);const b=v<<12|r;for(let i=0;i<18;i++){const a=size-11+i%3,c=Math.floor(i/3);set(a,c,bit(b,i));set(c,a,bit(b,i))}}
  // codewords
  let i=0;for(let right=size-1;right>=1;right-=2){if(right===6)right=5;for(let vert=0;vert<size;vert++)for(let j=0;j<2;j++){const x=right-j,up=((right+1)&2)===0,y=up?size-1-vert:vert;
    if(!F[y][x]&&i<cw.length*8){M[y][x]=bit(cw[i>>>3],7-(i&7));i++}}}
  const maskFn=[(x,y)=>(x+y)%2===0,(x,y)=>y%2===0,(x,y)=>x%3===0,(x,y)=>(x+y)%3===0,(x,y)=>(Math.floor(x/3)+Math.floor(y/2))%2===0,(x,y)=>x*y%2+x*y%3===0,(x,y)=>(x*y%2+x*y%3)%2===0,(x,y)=>((x+y)%2+x*y%3)%2===0];
  const apply=m=>{for(let y=0;y<size;y++)for(let x=0;x<size;x++)if(!F[y][x]&&maskFn[m](x,y))M[y][x]=!M[y][x]};
  const penalty=()=>{let p=0;const line=get=>{let run=1,s=0;const seq=[];for(let k=0;k<size;k++)seq.push(get(k));
      for(let k=1;k<=size;k++){if(k<size&&seq[k]===seq[k-1])run++;else{if(run>=5)s+=3+run-5;run=1}}
      for(let k=0;k+7<=size;k++){const w=seq.slice(k,k+7).map(Number).join('');if(w==='1011101'){const L=seq.slice(Math.max(0,k-4),k),R=seq.slice(k+7,k+11);if((L.length===4&&!L.some(Boolean))||(R.length===4&&!R.some(Boolean)))s+=40}}return s};
    for(let y=0;y<size;y++)p+=line(k=>M[y][k]);for(let x=0;x<size;x++)p+=line(k=>M[k][x]);
    let dark=0;for(let y=0;y<size;y++)for(let x=0;x<size;x++){if(M[y][x])dark++;if(x<size-1&&y<size-1){const c=M[y][x];if(c===M[y][x+1]&&c===M[y+1][x]&&c===M[y+1][x+1])p+=3}}
    p+=Math.floor(Math.abs(dark*20-size*size*10)/(size*size))*10;return p};
  let best=forceMask;
  if(best==null){let bp=Infinity;for(let m=0;m<8;m++){apply(m);drawFormat(m);const p=penalty();if(p<bp){bp=p;best=m}apply(m)}}
  apply(best);drawFormat(best);
  return {size,modules:M,version:v,mask:best};
}
/* SVG path for the matrix, with a quiet zone of `qz` modules */
function svg(text,opt){opt=opt||{};const q=encode(text,opt.ecl||'M'),qz=opt.quiet==null?2:opt.quiet,n=q.size+qz*2;let d='';
  for(let y=0;y<q.size;y++)for(let x=0;x<q.size;x++)if(q.modules[y][x])d+=`M${x+qz},${y+qz}h1v1h-1z`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${n} ${n}" shape-rendering="crispEdges" role="img" aria-label="${opt.label||'QR code'}"><rect width="${n}" height="${n}" fill="${opt.bg||'#f4f3f0'}"/><path d="${d}" fill="${opt.fg||'#000'}"/></svg>`}
g.QR={encode,svg};
if(typeof module!=='undefined')module.exports={encode,svg};
})(typeof window!=='undefined'?window:globalThis);
