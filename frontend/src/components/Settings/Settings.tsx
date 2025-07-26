import { useState, useEffect } from 'react'
import { User } from '../../App'
import VoiceCheck from '../VoiceCheck/VoiceCheck'
import styles from './Settings.module.css'

interface SettingsProps {
  currentUser: User | null
  onSave: (settings: any) => void
  onCancel: () => void
}

interface AudioDevice {
  deviceId: string
  label: string
}

function Settings({ currentUser, onSave, onCancel }: SettingsProps) {
  const [userName, setUserName] = useState(currentUser?.name || '')
  const [avatar, setAvatar] = useState(currentUser?.avatar || '')
  const [inputDevices, setInputDevices] = useState<AudioDevice[]>([])
  const [outputDevices, setOutputDevices] = useState<AudioDevice[]>([])
  const [selectedInputDevice, setSelectedInputDevice] = useState(
    currentUser?.audioInputDevice || ''
  )
  const [selectedOutputDevice, setSelectedOutputDevice] = useState(
    currentUser?.audioOutputDevice || ''
  )

  useEffect(() => {
    loadAudioDevices()
  }, [])

  const loadAudioDevices = async () => {
    try {
      // Request microphone permission
      await navigator.mediaDevices.getUserMedia({ audio: true })
      
      const devices = await navigator.mediaDevices.enumerateDevices()
      
      const audioInputs = devices
        .filter(device => device.kind === 'audioinput')
        .map(device => ({
          deviceId: device.deviceId,
          label: device.label || `Microphone ${device.deviceId.slice(0, 8)}`
        }))
      
      const audioOutputs = devices
        .filter(device => device.kind === 'audiooutput')
        .map(device => ({
          deviceId: device.deviceId,
          label: device.label || `Speaker ${device.deviceId.slice(0, 8)}`
        }))

      setInputDevices(audioInputs)
      setOutputDevices(audioOutputs)
    } catch (error) {
      console.error('Failed to load audio devices:', error)
    }
  }

  const handleSave = () => {
    const settings = {
      name: userName.trim(),
      avatar: avatar.trim() || null,
      audioInputDevice: selectedInputDevice || null,
      audioOutputDevice: selectedOutputDevice || null
    }

    onSave(settings)
  }


  return (
    <div className={styles.settingsOverlay}>
      <div className={styles.settingsModal}>
        <div className={styles.header}>
          <h2>Settings</h2>
          <button className={styles.closeButton} onClick={onCancel}>
            ✕
          </button>
        </div>

        <div className={styles.content}>
          <div className={styles.section}>
            <h3>User Profile</h3>
            <div className={styles.formGroup}>
              <label htmlFor="userName">Display Name</label>
              <input
                id="userName"
                type="text"
                value={userName}
                onChange={(e) => setUserName(e.target.value)}
                placeholder="Enter your display name"
              />
            </div>
            
            <div className={styles.formGroup}>
              <label htmlFor="avatar">Avatar URL (Optional)</label>
              <input
                id="avatar"
                type="url"
                value={avatar}
                onChange={(e) => setAvatar(e.target.value)}
                placeholder="https://example.com/avatar.jpg"
              />
            </div>
          </div>

          <div className={styles.section}>
            <h3>Audio Settings</h3>
            
            <div className={styles.formGroup}>
              <label htmlFor="inputDevice">Input Device (Microphone)</label>
              <select
                id="inputDevice"
                value={selectedInputDevice}
                onChange={(e) => setSelectedInputDevice(e.target.value)}
              >
                <option value="">Default</option>
                {inputDevices.map(device => (
                  <option key={device.deviceId} value={device.deviceId}>
                    {device.label}
                  </option>
                ))}
              </select>
            </div>

            <div className={styles.formGroup}>
              <label htmlFor="outputDevice">Output Device (Speakers)</label>
              <select
                id="outputDevice"
                value={selectedOutputDevice}
                onChange={(e) => setSelectedOutputDevice(e.target.value)}
              >
                <option value="">Default</option>
                {outputDevices.map(device => (
                  <option key={device.deviceId} value={device.deviceId}>
                    {device.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Voice Check Component */}
            <VoiceCheck 
              selectedInputDevice={selectedInputDevice}
              selectedOutputDevice={selectedOutputDevice}
            />
          </div>
        </div>

        <div className={styles.footer}>
          <button className={styles.cancelButton} onClick={onCancel}>
            Cancel
          </button>
          <button 
            className={styles.saveButton} 
            onClick={handleSave}
            disabled={!userName.trim()}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  )
}

export default Settings