import { useState, useEffect, useRef } from 'react'
import styles from './VoiceCheck.module.css'

interface VoiceCheckProps {
  selectedInputDevice?: string
  selectedOutputDevice?: string
}

function VoiceCheck({ selectedInputDevice, selectedOutputDevice }: VoiceCheckProps) {
  const [isRecording, setIsRecording] = useState(false)
  const [volumeLevel, setVolumeLevel] = useState(0)
  const [hasRecording, setHasRecording] = useState(false)
  const [isPlaying, setIsPlaying] = useState(false)

  const mediaRecorderRef = useRef<MediaRecorder | null>(null)
  const audioContextRef = useRef<AudioContext | null>(null)
  const analyserRef = useRef<AnalyserNode | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const recordedChunksRef = useRef<Blob[]>([])
  const recordedAudioRef = useRef<HTMLAudioElement | null>(null)
  const animationFrameRef = useRef<number>()

  useEffect(() => {
    // Start monitoring volume level when component mounts
    startVolumeMonitoring()

    return () => {
      // Cleanup on unmount
      stopVolumeMonitoring()
      if (recordedAudioRef.current) {
        recordedAudioRef.current.pause()
        recordedAudioRef.current = null
      }
    }
  }, [selectedInputDevice])

  const startVolumeMonitoring = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          deviceId: selectedInputDevice ? { exact: selectedInputDevice } : undefined,
        },
      })

      streamRef.current = stream

      const audioContext = new AudioContext()
      const analyser = audioContext.createAnalyser()
      const microphone = audioContext.createMediaStreamSource(stream)

      analyser.fftSize = 256
      analyser.smoothingTimeConstant = 0.8

      microphone.connect(analyser)

      audioContextRef.current = audioContext
      analyserRef.current = analyser

      updateVolumeLevel()
    } catch (error) {
      console.error('Failed to start volume monitoring:', error)
    }
  }

  const stopVolumeMonitoring = () => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current)
    }

    if (streamRef.current) {
      streamRef.current.getTracks().forEach(track => track.stop())
      streamRef.current = null
    }

    if (audioContextRef.current) {
      audioContextRef.current.close()
      audioContextRef.current = null
    }

    analyserRef.current = null
    setVolumeLevel(0)
  }

  const updateVolumeLevel = () => {
    if (!analyserRef.current) return

    const bufferLength = analyserRef.current.frequencyBinCount
    const dataArray = new Uint8Array(bufferLength)

    analyserRef.current.getByteFrequencyData(dataArray)

    // Calculate average volume
    let sum = 0
    for (let i = 0; i < bufferLength; i++) {
      sum += dataArray[i]
    }
    const average = sum / bufferLength

    // Normalize to 0-100 range and apply some scaling for better visualization
    const normalizedVolume = Math.min(100, (average / 255) * 200)
    setVolumeLevel(normalizedVolume)

    animationFrameRef.current = requestAnimationFrame(updateVolumeLevel)
  }

  const startRecording = async () => {
    try {
      if (!streamRef.current) {
        await startVolumeMonitoring()
      }

      if (!streamRef.current) {
        throw new Error('Failed to get audio stream')
      }

      recordedChunksRef.current = []

      const mediaRecorder = new MediaRecorder(streamRef.current, {
        mimeType: 'audio/webm;codecs=opus',
      })

      mediaRecorder.ondataavailable = event => {
        if (event.data.size > 0) {
          recordedChunksRef.current.push(event.data)
        }
      }

      mediaRecorder.onstop = () => {
        const blob = new Blob(recordedChunksRef.current, { type: 'audio/webm;codecs=opus' })
        const audioUrl = URL.createObjectURL(blob)

        if (recordedAudioRef.current) {
          recordedAudioRef.current.pause()
          URL.revokeObjectURL(recordedAudioRef.current.src)
        }

        recordedAudioRef.current = new Audio(audioUrl)

        if (selectedOutputDevice && recordedAudioRef.current.setSinkId) {
          recordedAudioRef.current.setSinkId(selectedOutputDevice).catch(console.error)
        }

        setHasRecording(true)
      }

      mediaRecorderRef.current = mediaRecorder
      mediaRecorder.start()
      setIsRecording(true)
    } catch (error) {
      console.error('Failed to start recording:', error)
    }
  }

  const stopRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop()
      mediaRecorderRef.current = null
      setIsRecording(false)
    }
  }

  const playRecording = () => {
    if (recordedAudioRef.current && hasRecording) {
      recordedAudioRef.current.currentTime = 0

      recordedAudioRef.current.onended = () => setIsPlaying(false)
      recordedAudioRef.current.onerror = () => setIsPlaying(false)

      recordedAudioRef.current
        .play()
        .then(() => setIsPlaying(true))
        .catch(error => {
          console.error('Failed to play recording:', error)
          setIsPlaying(false)
        })
    }
  }

  const stopPlayback = () => {
    if (recordedAudioRef.current) {
      recordedAudioRef.current.pause()
      recordedAudioRef.current.currentTime = 0
      setIsPlaying(false)
    }
  }

  return (
    <div className={styles.voiceCheck}>
      <h4>Voice Check</h4>

      {/* Volume Level Meter */}
      <div className={styles.volumeMeter}>
        <label>Microphone Level</label>
        <div className={styles.volumeBar}>
          <div
            className={styles.volumeLevel}
            style={{
              width: `${volumeLevel}%`,
              backgroundColor:
                volumeLevel > 80 ? '#ff4444' : volumeLevel > 50 ? '#ffaa00' : '#44ff44',
            }}
          />
        </div>
        <span className={styles.volumeText}>{Math.round(volumeLevel)}%</span>
      </div>

      {/* Recording Controls */}
      <div className={styles.recordingControls}>
        <button
          className={`${styles.recordButton} ${isRecording ? styles.recording : ''}`}
          onClick={isRecording ? stopRecording : startRecording}
        >
          {isRecording ? '⏹️ Stop' : '🎤 Record'}
        </button>

        <button
          className={`${styles.playButton} ${!hasRecording ? styles.disabled : ''}`}
          onClick={isPlaying ? stopPlayback : playRecording}
          disabled={!hasRecording}
        >
          {isPlaying ? '⏸️ Stop' : '▶️ Play'}
        </button>
      </div>

      {isRecording && (
        <div className={styles.recordingIndicator}>
          <span className={styles.recordingDot}></span>
          Recording...
        </div>
      )}

      {isPlaying && <div className={styles.playingIndicator}>Playing recorded audio...</div>}
    </div>
  )
}

export default VoiceCheck
