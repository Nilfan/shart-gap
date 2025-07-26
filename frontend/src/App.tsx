import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import ChatArea from './components/ChatArea/ChatArea'
import Settings from './components/Settings/Settings'
import PartyMembers from './components/PartyMembers/PartyMembers'
import styles from './App.module.css'

export interface Party {
  id: string
  name: string
  creator_id: string
  users: { [key: string]: User }
  messages: Message[]
  server_user_id?: string
  protocol: string
  peer_addresses: string[]
  ping_measurements: { [key: string]: number }
  created_at: string
  is_voice_enabled: boolean
  invite_code?: string
  // UI-only fields
  isConnected?: boolean
  hasActiveCall?: boolean
}

export interface User {
  id: string
  name: string
  avatar?: string
  isOnline: boolean
  audioInputDevice?: string
  audioOutputDevice?: string
  isInCall?: boolean
}

export interface Message {
  id: string
  userId: string
  userName: string
  content: string
  timestamp: string
}

function App() {
  const [currentParty, setCurrentParty] = useState<Party | null>(null)
  const [showSettings, setShowSettings] = useState(false)
  const [currentUser, setCurrentUser] = useState<User | null>(null)
  const [userName, setUserName] = useState('')
  const [isUserInitialized, setIsUserInitialized] = useState(false)
  const [showJoinModal, setShowJoinModal] = useState(false)
  const [inviteCode, setInviteCode] = useState('')
  const [isMuted, setIsMuted] = useState(false)
  const [isInCall, setIsInCall] = useState(false)
  const [userIP, setUserIP] = useState<string>('')
  const [isJoining, setIsJoining] = useState(false)
  const [joinStatus, setJoinStatus] = useState<string>('')

  useEffect(() => {
    // Check for saved name on app start
    const savedName = localStorage.getItem('shortgap-username')
    if (savedName) {
      initializeUser(savedName)
    }
    
    // Get user IP address
    const fetchUserIP = async () => {
      try {
        const ip = await invoke<string>('get_user_ip')
        setUserIP(ip)
      } catch (error) {
        console.error('Failed to get user IP:', error)
      }
    }
    
    fetchUserIP()
  }, [])

  const initializeUser = async (name: string) => {
    try {
      const userId = `user-${Date.now()}-${Math.floor(Math.random() * 10000)}`
      const user: User = {
        id: userId,
        name: name,
        avatar: undefined,
        isOnline: true,
        audioInputDevice: undefined,
        audioOutputDevice: undefined,
      }

      await invoke('set_user_settings', {
        settings: {
          name: user.name,
          avatar: user.avatar,
          audioInputDevice: user.audioInputDevice,
          audioOutputDevice: user.audioOutputDevice,
        },
      })

      setCurrentUser(user)
      setIsUserInitialized(true)
    } catch (error) {
      console.error('Failed to initialize user:', error)
    }
  }

  const handleStartParty = async () => {
    try {
      const newParty = await invoke<Party>('create_party', { name: 'Party' })
      const inviteCode = await invoke<string>('generate_invite')

      setCurrentParty({ ...newParty, invite_code: inviteCode })
    } catch (error) {
      console.error('Failed to start party:', error)
      alert(`Failed to start party: ${error}`)
    }
  }

  const handleJoinParty = async (inviteCode: string) => {
    if (!inviteCode.trim()) return
    
    setIsJoining(true)
    setJoinStatus('Validating invite code...')
    
    try {
      // First validate the invite
      const validationResult = await invoke<string>('validate_invite', { inviteCode })
      console.log('Invite validation:', validationResult)
      setJoinStatus('Invite valid! Connecting to party...')
      
      // Small delay to show validation message
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Attempt to join
      setJoinStatus('Establishing connection...')
      const joinedParty = await invoke<Party>('join_party', { inviteCode })
      
      setJoinStatus('Connected! Joining party...')
      setCurrentParty(joinedParty)
      setShowJoinModal(false)
      setInviteCode('')
      setJoinStatus('')
    } catch (error) {
      console.error('Failed to join party:', error)
      setJoinStatus('')
      alert(`Failed to join party: ${error}`)
    } finally {
      setIsJoining(false)
    }
  }

  const handleSendMessage = async (content: string) => {
    if (!currentParty) return

    try {
      await invoke('send_message', {
        content,
      })

      await refreshCurrentParty()
    } catch (error) {
      console.error('Failed to send message:', error)
    }
  }

  const refreshCurrentParty = async () => {
    try {
      const updatedParty = await invoke<Party | null>('get_current_party')
      if (updatedParty) {
        setCurrentParty({ ...updatedParty, invite_code: currentParty?.invite_code })
      }
    } catch (error) {
      console.error('Failed to refresh party:', error)
    }
  }

  const handleLeaveParty = async () => {
    try {
      await invoke('leave_party')
      setCurrentParty(null)
      setIsInCall(false)
    } catch (error) {
      console.error('Failed to leave party:', error)
      // Still set local state even if backend fails
      setCurrentParty(null)
      setIsInCall(false)
    }
  }

  const handleSaveSettings = async (settings: any) => {
    try {
      await invoke('set_user_settings', { settings })
      setShowSettings(false)
    } catch (error) {
      console.error('Failed to save settings:', error)
    }
  }

  const handleSubmitName = () => {
    if (userName.trim()) {
      const name = userName.trim()
      localStorage.setItem('shortgap-username', name)
      initializeUser(name)
    }
  }

  const handleToggleMute = () => {
    setIsMuted(!isMuted)
    // TODO: Implement actual mute/unmute logic with backend
  }

  const handleJoinCall = async () => {
    if (!currentParty) return

    try {
      await invoke('join_call')
      setIsInCall(true)
      await refreshCurrentParty()
    } catch (error) {
      console.error('Failed to join call:', error)
      alert(`Failed to join call: ${error}`)
    }
  }

  const handleLeaveCall = async () => {
    if (!currentParty) return

    try {
      await invoke('leave_call')
      setIsInCall(false)
      await refreshCurrentParty()
    } catch (error) {
      console.error('Failed to leave call:', error)
      alert(`Failed to leave call: ${error}`)
    }
  }

  // Name input screen
  if (!isUserInitialized) {
    return (
      <div className={styles.app}>
        <div className={styles.topBar}>
          <div className={styles.userInfo}>
            {userIP && <span className={styles.ipInfo}>IP: {userIP}</span>}
          </div>
          <div className={styles.topBarActions}>
            <button onClick={() => setShowSettings(true)} className={styles.settingsButton}>
              Settings
            </button>
          </div>
        </div>
        <div className={styles.nameScreen}>
          <h2>Welcome to ShortGap</h2>
          <p>Enter your name to continue</p>
          <div className={styles.nameInput}>
            <input
              type='text'
              value={userName}
              onChange={e => setUserName(e.target.value)}
              placeholder='Your name'
              onKeyDown={e => e.key === 'Enter' && handleSubmitName()}
            />
            <button onClick={handleSubmitName} disabled={!userName.trim()}>
              Continue
            </button>
          </div>
        </div>
        {showSettings && (
          <Settings
            currentUser={null}
            onSave={handleSaveSettings}
            onCancel={() => setShowSettings(false)}
          />
        )}
      </div>
    )
  }

  // Main party interface
  return (
    <div className={styles.app}>
      <div className={styles.topBar}>
        <div className={styles.userInfo}>
          <span>{currentUser?.name}</span>
          {userIP && <span className={styles.ipInfo}>IP: {userIP}</span>}
        </div>
        <div className={styles.topBarActions}>
          {currentParty && (
            <>
              {currentParty.invite_code && (
                <button
                  onClick={() => navigator.clipboard.writeText(currentParty.invite_code!)}
                  className={styles.copyInviteButton}
                >
                  Copy invite link
                </button>
              )}
              <button onClick={handleLeaveParty} className={styles.leaveButton}>
                Leave Party
              </button>
            </>
          )}
        </div>
      </div>

      <div className={styles.contentWrapper}>
        {currentParty && (
          <PartyMembers
            party={currentParty}
            currentUser={currentUser}
            isMuted={isMuted}
            onToggleMute={handleToggleMute}
            onOpenSettings={() => setShowSettings(true)}
          />
        )}

        <div className={styles.mainContent}>
          {currentParty ? (
            <ChatArea
              room={currentParty}
              currentUser={currentUser}
              isInCall={isInCall}
              onJoinCall={handleJoinCall}
              onLeaveCall={handleLeaveCall}
              onSendMessage={handleSendMessage}
            />
          ) : (
            <div className={styles.welcomeScreen}>
              <h2>Ready to party?</h2>
              <div className={styles.partyButtons}>
                <button className={styles.partyButton} onClick={handleStartParty}>
                  Start a party
                </button>
                <button className={styles.partyButton} onClick={() => setShowJoinModal(true)}>
                  Join a party
                </button>
                <button className={styles.settingsButton} onClick={() => setShowSettings(true)}>
                  Settings
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {showJoinModal && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <h3>Join Party</h3>
            <input
              type='text'
              value={inviteCode}
              onChange={e => setInviteCode(e.target.value)}
              placeholder='Enter invite code'
              disabled={isJoining}
            />
            {joinStatus && (
              <div className={styles.statusMessage}>
                {joinStatus}
              </div>
            )}
            <div className={styles.modalButtons}>
              <button 
                onClick={() => handleJoinParty(inviteCode)} 
                disabled={!inviteCode.trim() || isJoining}
              >
                {isJoining ? 'Joining...' : 'Join'}
              </button>
              <button 
                onClick={() => {
                  if (!isJoining) {
                    setShowJoinModal(false)
                    setInviteCode('')
                    setJoinStatus('')
                  }
                }}
                disabled={isJoining}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {showSettings && (
        <Settings
          currentUser={currentUser}
          onSave={handleSaveSettings}
          onCancel={() => setShowSettings(false)}
        />
      )}
    </div>
  )
}

export default App
