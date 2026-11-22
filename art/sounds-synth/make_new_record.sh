#!/bin/bash
# 폭죽 소리 (휘파람 -> 퐁 -> 반짝 흩어짐). ffmpeg aevalsrc 합성, CC0.
# 사용: bash art/sounds-synth/make_new_record.sh  (프로젝트 폴더에서)
OUT=game/assets/sounds/sfx/new_record.wav
PI=3.14159265
ARGS=(); FC=""; MIX=""; n=0
add() { # 식 길이 지연ms 추가필터
  local bs=$'\\'; local e="${1//,/$bs,}"
  ARGS+=(-f lavfi -i "aevalsrc=exprs=${e}:d=$2:s=22050")
  FC+="[$n:a]${4}${4:+,}adelay=$3:all=1[a$n];"
  MIX+="[a$n]"; n=$((n+1))
}
# 1) 올라가는 휘파람 0.35초, 600 -> 1800Hz
add "0.3*sin(2*$PI*(600*t+1714.3*t*t))*min(1,t/0.06)*min(1,(0.35-t)/0.03)" 0.35 0 ""
# 2) 퐁: 고음 노이즈 0.08초(2kHz 이상) + C6 E6 G6 화음 0.25초
add "0.22*(random(0)*2-1)*exp(-t*45)" 0.08 350 "highpass=f=2000"
add "0.12*(sin(2*$PI*1046.5*t)+sin(2*$PI*1318.5*t)+sin(2*$PI*1568*t))*exp(-t*9)*min(1,t/0.004)" 0.25 350 ""
# 3) 반짝 8개 (2~4kHz, 간격 점점 넓어지고 점점 작아짐)
for spec in "0 3300 0.22" "70 2400 0.2" "150 3800 0.17" "250 2800 0.15" "370 3500 0.12" "510 2200 0.1" "670 3900 0.08" "850 3000 0.06"; do
  read ms hz amp <<< "$spec"
  add "$amp*sin(2*$PI*$hz*t)*sin($PI*t/0.045)" 0.045 $((450+ms)) ""
done
ffmpeg -y -v error "${ARGS[@]}" -filter_complex "${FC}${MIX}amix=inputs=$n:normalize=0,volume=0.9,apad=whole_dur=1.5,atrim=0:1.5,afade=t=in:d=0.005,afade=t=out:st=1.4:d=0.1" -ac 1 -ar 22050 -c:a pcm_s16le "$OUT"
