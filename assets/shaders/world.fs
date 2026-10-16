#version 330
in vec3 worldPos;
in vec3 worldNormal;
in vec3 localPos;
uniform vec4 colDiffuse;
uniform vec3 eyePos;
uniform int bodyKind;
uniform float simTime;
out vec4 finalColor;

float hash(vec3 p) { return fract(sin(dot(p,vec3(127.1,311.7,74.7)))*43758.5453); }
float noise(vec3 p) {
    vec3 i=floor(p),f=fract(p);f=f*f*(3.0-2.0*f);
    return mix(mix(mix(hash(i),hash(i+vec3(1,0,0)),f.x),mix(hash(i+vec3(0,1,0)),hash(i+vec3(1,1,0)),f.x),f.y),
        mix(mix(hash(i+vec3(0,0,1)),hash(i+vec3(1,0,1)),f.x),mix(hash(i+vec3(0,1,1)),hash(i+vec3(1,1,1)),f.x),f.y),f.z);
}
float fbm(vec3 p) {
    float s=0.0,a=0.55;
    for(int i=0;i<4;i++){s+=a*noise(p);p=p*2.07+vec3(5.2,1.3,3.8);a*=0.5;}
    return s;
}
void main() {
    vec3 p=normalize(localPos),n=normalize(worldNormal);
    float f=fbm(p*5.0),bands=sin(p.y*65.0+fbm(p*10.0)*5.0);
    vec3 albedo=colDiffuse.rgb,atmosphere=vec3(0);
    if(bodyKind==0){
        float heat=fbm(p*13.0+simTime*0.08);
        vec3 fire=mix(vec3(1.0,0.24,0.025),vec3(1.0,0.91,0.35),smoothstep(0.25,0.8,heat));
        float edge=pow(max(dot(n,normalize(eyePos-worldPos)),0.0),0.3);
        finalColor=vec4(fire*(0.72+0.28*edge),1);return;
    }
    if(bodyKind==1 || bodyKind==10){
        float craters=smoothstep(0.63,0.72,noise(p*38.0));
        albedo=mix(vec3(0.22,0.21,0.22),vec3(0.66,0.62,0.54),f)*(1.0-craters*0.45);
    }
    if(bodyKind==2){albedo=mix(vec3(0.63,0.31,0.10),vec3(1.0,0.84,0.50),fbm(p*4.0+vec3(bands*0.2)));atmosphere=vec3(0.5,0.24,0.08);}
    if(bodyKind==3){
        float continent=fbm(p*3.0+vec3(2,6,3));
        vec3 land=mix(vec3(0.08,0.27,0.10),vec3(0.56,0.55,0.27),fbm(p*11.0));
        albedo=mix(vec3(0.015,0.14,0.44),land,smoothstep(0.49,0.53,continent));
        float cloud=smoothstep(0.63,0.76,fbm(p*7.0+vec3(simTime*0.01,0,0)));
        albedo=mix(albedo,vec3(0.93,0.97,1.0),cloud*0.85);
        albedo=mix(albedo,vec3(0.84,0.94,0.98),smoothstep(0.82,0.94,abs(p.y)+f*0.08));
        atmosphere=vec3(0.12,0.49,0.95);
    }
    if(bodyKind==4){albedo=mix(vec3(0.28,0.09,0.055),vec3(0.89,0.37,0.15),f);albedo=mix(albedo,vec3(0.82,0.73,0.62),smoothstep(0.95,0.99,abs(p.y)));}
    if(bodyKind==5){
        float band=sin(p.y*36.0+fbm(p*9.0)*4.0);
        albedo=mix(vec3(0.57,0.29,0.15),vec3(0.96,0.85,0.67),band*0.5+0.5);
        float spot=length((p.xy-vec2(0.4,-0.24))*vec2(4.0,8.0));
        if(p.z>0.0) albedo=mix(albedo,vec3(0.66,0.22,0.12),(1.0-smoothstep(0.6,1.0,spot))*0.8);
    }
    if(bodyKind==6){albedo=mix(vec3(0.62,0.47,0.27),vec3(0.94,0.85,0.61),0.5+0.25*bands+f*0.25);}
    if(bodyKind==7){albedo=mix(vec3(0.24,0.59,0.64),vec3(0.65,0.90,0.90),0.5+bands*0.05+f*0.2);atmosphere=vec3(0.22,0.60,0.66);}
    if(bodyKind==8){albedo=mix(vec3(0.035,0.13,0.54),vec3(0.16,0.38,0.91),f+bands*0.12);atmosphere=vec3(0.10,0.23,0.70);}
    vec3 light=normalize(-worldPos);
    float diffuse=max(dot(n,light),0.0);
    // Relleno artistico suave para leer las superficies incluso en el hemisferio nocturno.
    float ambient=bodyKind==9?0.50:0.27;
    float fill=0.12*max(dot(n,normalize(eyePos-worldPos)),0.0);
    vec3 color=albedo*min(1.0,ambient+(1.0-ambient)*diffuse+fill);
    float rim=pow(1.0-max(dot(n,normalize(eyePos-worldPos)),0.0),3.2);
    color+=atmosphere*rim*0.48;
    finalColor=vec4(color,1.0);
}
