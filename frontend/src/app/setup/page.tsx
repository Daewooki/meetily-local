'use client';
import { LocalSetup } from '@/components/LocalSetup';
export default function SetupPage(){return <LocalSetup onComplete={()=>{window.location.href='/connections';}}/>;}
