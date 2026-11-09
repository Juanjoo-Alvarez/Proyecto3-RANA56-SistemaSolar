"""Build the class trailer from actual in-engine captures. Requires Pillow, NumPy, FFmpeg."""
from pathlib import Path
import subprocess, wave, json
import numpy as np
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parents[1]
WORK=ROOT/'target/trailer'; WORK.mkdir(parents=True,exist_ok=True)
OUT=ROOT/'output/video'; OUT.mkdir(parents=True,exist_ok=True)
FF=ROOT/'target/video-tools/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v9.0.2.exe'
W,H=1440,810
FONT=ROOT/'assets/fonts'
def font(n,bold=False): return ImageFont.truetype(str(FONT/('Nunito-ExtraBold.ttf' if bold else 'Nunito-SemiBold.ttf')),n)
# source, in point, seconds, eyebrow, headline, composition
SHOTS=[
 ('space',0,4,'EN ALGÚN LUGAR DEL SISTEMA SOLAR','Una pequeña nave.','intro'),
 ('space',5,6,'EXPLORACIÓN ESPACIAL','Un universo por explorar.','lower'),
 ('space',21,5,'ACÉRCATE · RODEA · EXPLORA','Cada mundo, una nueva perspectiva.','lower'),
 ('space',29,3,'UNA AVENTURA CON MUCHO CORAZÓN','Operación Croac','center'),
 ('mercury',2,7,'MERCURIO','Atrévete a cruzar el calor.','lower'),
 ('mercury',10,5,'MISIÓN DE RESCATE','Encuentra a Brasa.','lower'),
 ('brasa',0,4.5,'NO ESTÁS SOLO','Ninguna rana se queda atrás.','lower'),
 ('venus',3,7.5,'VENUS · JARDINES DE NIMBO','Salta hacia lo desconocido.','lower'),
 ('venus',12,4,'RECUPERA LAS SEMILLAS DE LUZ','La esperanza también florece aquí.','lower'),
 ('lirio',0,4.5,'UN NUEVO AMIGO','Trae a Lirio de vuelta.','lower'),
 ('crew',0,5.5,'DE VUELTA A CASA','Tu tripulación crece contigo.','lower'),
 ('space',38,6,'','RANA / 56','end'),
]
DURATION=sum(s[2] for s in SHOTS)
def overlay(idx,eyebrow,title,style):
    im=Image.new('RGBA',(W,H)); d=ImageDraw.Draw(im)
    if style=='lower':
        for y in range(480,H):
            a=int(205*((y-480)/(H-480))**1.2)
            d.line((0,y,W,y),fill=(5,14,22,a))
        d.line((78,626,126,626),fill=(130,241,202,255),width=4)
        d.text((78,649),eyebrow,font=font(21,True),fill=(153,239,210,255))
        d.text((76,684),title,font=font(49,True),fill=(250,247,230,255),stroke_width=1)
    else:
        d.rectangle((0,0,W,H),fill=(5,14,24,130 if style!='end' else 175))
        def centered(text,y,f,color):
            box=d.textbbox((0,0),text,font=f); x=(W-(box[2]-box[0]))/2
            d.text((x,y),text,font=f,fill=color)
        mint=(153,239,210,255); cream=(250,247,230,255)
        if style=='end':
            centered(title,230,font(116,True),cream)
            centered('Operación Croac',367,font(49),mint)
            d.line((650,456,790,456),fill=mint,width=3)
            centered('Explora. Rescata. Vuelve a casa.',485,font(31),cream)
            centered('Juan Jose Rivas Alvarez  ·  24856',657,font(23),cream)
            centered('Proyecto de Gráficos por Computadora',692,font(20),mint)
            centered('Imágenes del juego · Captura por fotogramas · Versión en desarrollo',760,font(16),(177,192,197,255))
        else:
            centered(eyebrow,307,font(21,True),mint)
            centered(title,352,font(71,True),cream)
            d.line((680,468,760,468),fill=mint,width=3)
    path=WORK/f'text-{idx:02}.png';im.save(path);return path

def run(args): subprocess.run([str(FF),'-hide_banner','-loglevel','error','-y',*map(str,args)],check=True,cwd=ROOT)

def soundtrack():
    sr=32000; mix=np.zeros((int(DURATION*sr),2),np.float64)
    def add(name,start,duration,gain=1,offset=0,fade=1.5):
        with wave.open(str(ROOT/'assets/audio'/name),'rb') as wav:
            assert wav.getframerate()==sr and wav.getnchannels()==2
            data=np.frombuffer(wav.readframes(wav.getnframes()),'<i2').reshape(-1,2).astype(float)/32768
        count=min(int(duration*sr),len(mix)-int(start*sr)); begin=int(offset*sr)
        data=data[np.arange(begin,begin+count)%len(data)].copy()
        ramp=min(int(fade*sr),count//2);env=np.ones(count)
        env[:ramp]=np.linspace(0,1,ramp);env[-ramp:]=np.linspace(1,0,ramp)
        mix[int(start*sr):int(start*sr)+count]+=data*env[:,None]*gain
    add('space.wav',0,20,1.2,offset=12,fade=2)
    add('mercury.wav',17,18,1.25,offset=24,fade=2)
    add('lunar.wav',32,18,1.3,offset=16,fade=2)
    add('rescue.wav',47,12,1.25,fade=1)
    add('cabin.wav',56,DURATION-56,1.1,offset=12,fade=2)
    # Quiet original rhythmic accents underpin the action, without masking the game's music.
    rng=np.random.default_rng(24856)
    for beat,t0 in enumerate(np.arange(18,46,60/112)):
        t=np.arange(int(.32*sr))/sr
        pulse=(.045*np.sin(2*np.pi*(58*t+14*t*np.exp(-t*15)))*np.exp(-t*17)
               + .008*rng.standard_normal(len(t))*np.exp(-t*60))
        strength=.5+.5*min((t0-18)/18,1)
        pos=int(t0*sr);mix[pos:pos+len(t)]+=pulse[:,None]*strength
    peak=np.max(np.abs(mix)); mix*=min(1.7,.75/max(peak,1e-6))
    ramp=int(2*sr);mix[-ramp:]*=np.linspace(1,0,ramp)[:,None]
    path=WORK/'trailer-music.wav'
    with wave.open(str(path),'wb') as wav:
        wav.setnchannels(2);wav.setsampwidth(2);wav.setframerate(sr)
        wav.writeframes((np.clip(mix,-1,1)*32767).astype('<i2').tobytes())
    return path

def main():
    clips=[];timeline=[];cursor=0
    for i,(source,start,duration,brow,title,style) in enumerate(SHOTS):
        print(f'Montaje {i+1}/{len(SHOTS)}: {source} / {title}',flush=True)
        art=overlay(i,brow,title,style);clip=WORK/f'edit-{i:02}.mp4'
        filters=(f'[0:v]crop=1440:810:0:45,setsar=1,eq=contrast=1.035:saturation=1.06[v];'
                 f'[1:v]format=rgba,fade=t=in:st=0.25:d=0.45:alpha=1[title];'
                 f'[v][title]overlay=0:0:shortest=1,fade=t=in:st=0:d=0.18,'
                 f'fade=t=out:st={duration-.22}:d=0.22,format=yuv420p[out]')
        run(['-ss',start,'-i',WORK/f'{source}.mp4','-loop','1','-framerate','30','-i',art,
             '-filter_complex',filters,'-map','[out]','-an','-t',duration,'-r','30',
             '-c:v','libx264','-preset','fast','-crf','18','-threads','4',clip])
        clips.append(clip);timeline.append({'inicio':cursor,'duracion':duration,'toma':source,'texto':title});cursor+=duration
    concat=WORK/'edit-list.txt';concat.write_text('\n'.join("file '"+p.as_posix()+"'" for p in clips),encoding='utf-8')
    music=soundtrack()
    output=OUT/'RANA56-TRAILER.mp4'
    run(['-f','concat','-safe','0','-i',concat,'-i',music,'-map','0:v:0','-map','1:a:0',
         '-c:v','copy','-c:a','aac','-b:a','192k','-t',DURATION,'-movflags','+faststart',output])
    (WORK/'timeline.json').write_text(json.dumps(timeline,ensure_ascii=False,indent=2),encoding='utf-8')
    run(['-i',output,'-vf','fps=1/5,scale=360:202,tile=4x3','-frames:v','1','-update','1',WORK/'contact.png'])
    print(f'Trailer listo: {output} ({DURATION} segundos)',flush=True)
if __name__=='__main__': main()
