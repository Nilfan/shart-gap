import { Party, User } from '../../App'
import MicIcon from '@mui/icons-material/Mic'
import MicOffIcon from '@mui/icons-material/MicOff'
import styles from './PartyMembers.module.css'

interface PartyMembersProps {
  party: Party
  currentUser: User | null
  isMuted: boolean
  isInCall: boolean
  onToggleMute: () => void
  onJoinCall: () => void
  onLeaveCall: () => void
  onOpenSettings: () => void
}

function PartyMembers({ party, currentUser, isMuted, isInCall, onToggleMute, onJoinCall, onLeaveCall, onOpenSettings }: PartyMembersProps) {
  const members = Object.values(party.users)

  return (
    <div className={styles.partyMembers}>
      <div className={styles.header}>
        <h3>Party Members</h3>
        <span className={styles.memberCount}>{members.length}</span>
      </div>
      
      <div className={styles.membersList}>
        {members.map((member) => (
          <div key={member.id} className={styles.member}>
            <div className={styles.memberInfo}>
              <div className={styles.avatar}>
                {member.avatar ? (
                  <img src={member.avatar} alt={member.name} />
                ) : (
                  <div className={styles.defaultAvatar}>
                    {member.name.charAt(0).toUpperCase()}
                  </div>
                )}
              </div>
              <div className={styles.memberDetails}>
                <span className={styles.memberName}>
                  {member.name}
                  {currentUser?.id === member.id && (
                    <span className={styles.youIndicator}> (you)</span>
                  )}
                </span>
                <div className={styles.memberStatus}>
                  <div className={`${styles.statusDot} ${member.isOnline ? styles.online : styles.offline}`} />
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
      
      <div className={styles.controlButtons}>
        <button 
          className={`${styles.callButton} ${isInCall ? styles.inCall : styles.notInCall}`}
          onClick={isInCall ? onLeaveCall : onJoinCall}
        >
          {isInCall ? 'Leave call' : 'Join call'}
        </button>
        <div className={styles.bottomRow}>
          <button 
            className={styles.settingsButton} 
            onClick={onOpenSettings}
          >
            ⚙️ Settings
          </button>
          <button 
            className={`${styles.muteButton} ${isMuted ? styles.muted : ''}`} 
            onClick={onToggleMute}
            title={isMuted ? 'Unmute' : 'Mute'}
          >
            {isMuted ? <MicOffIcon /> : <MicIcon />}
          </button>
        </div>
      </div>
    </div>
  )
}

export default PartyMembers