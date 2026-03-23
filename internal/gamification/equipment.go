package gamification

import (
	"database/sql"
	"fmt"
)

// Equipped represents currently equipped items
type Equipped struct {
	HatID        string
	HeldID       string
	AuraID       string
	BackgroundID string
}

// GetEquipped returns currently equipped items
func GetEquipped(db *sql.DB) (*Equipped, error) {
	eq := &Equipped{}
	var hatID, heldID, auraID, bgID sql.NullString

	err := db.QueryRow(`
		SELECT hat_id, held_id, aura_id, background_id
		FROM equipped WHERE id = 'default'
	`).Scan(&hatID, &heldID, &auraID, &bgID)
	if err != nil {
		return nil, err
	}

	eq.HatID = hatID.String
	eq.HeldID = heldID.String
	eq.AuraID = auraID.String
	eq.BackgroundID = bgID.String

	return eq, nil
}

// EquipItem equips an item to the appropriate slot
func EquipItem(db *sql.DB, itemID string) error {
	// Check ownership
	if !OwnsItem(db, itemID) {
		return fmt.Errorf("item not owned: %s", itemID)
	}

	// Get item to determine slot
	item, err := GetShopItem(db, itemID)
	if err != nil {
		return fmt.Errorf("item not found: %s", itemID)
	}

	// Update appropriate slot
	var query string
	switch item.Slot {
	case SlotHat:
		query = `UPDATE equipped SET hat_id = ? WHERE id = 'default'`
	case SlotHeld:
		query = `UPDATE equipped SET held_id = ? WHERE id = 'default'`
	case SlotAura:
		query = `UPDATE equipped SET aura_id = ? WHERE id = 'default'`
	case SlotBackground:
		query = `UPDATE equipped SET background_id = ? WHERE id = 'default'`
	default:
		return fmt.Errorf("unknown slot: %s", item.Slot)
	}

	_, err = db.Exec(query, itemID)
	return err
}

// UnequipSlot removes item from a slot
func UnequipSlot(db *sql.DB, slot string) error {
	var query string
	switch slot {
	case SlotHat:
		query = `UPDATE equipped SET hat_id = NULL WHERE id = 'default'`
	case SlotHeld:
		query = `UPDATE equipped SET held_id = NULL WHERE id = 'default'`
	case SlotAura:
		query = `UPDATE equipped SET aura_id = NULL WHERE id = 'default'`
	case SlotBackground:
		// Background defaults back to default
		query = `UPDATE equipped SET background_id = 'bg_default' WHERE id = 'default'`
	default:
		return fmt.Errorf("unknown slot: %s", slot)
	}

	_, err := db.Exec(query)
	return err
}

// GetEquippedCount returns number of equipped accessory slots (excludes background)
func GetEquippedCount(db *sql.DB) int {
	eq, err := GetEquipped(db)
	if err != nil {
		return 0
	}

	count := 0
	if eq.HatID != "" {
		count++
	}
	if eq.HeldID != "" {
		count++
	}
	if eq.AuraID != "" {
		count++
	}
	return count
}

// GetEquippedItems returns full item details for equipped items
func GetEquippedItems(db *sql.DB) (map[string]*ShopItem, error) {
	eq, err := GetEquipped(db)
	if err != nil {
		return nil, err
	}

	result := make(map[string]*ShopItem)

	if eq.HatID != "" {
		if item, err := GetShopItem(db, eq.HatID); err == nil {
			result["hat"] = item
		}
	}
	if eq.HeldID != "" {
		if item, err := GetShopItem(db, eq.HeldID); err == nil {
			result["held"] = item
		}
	}
	if eq.AuraID != "" {
		if item, err := GetShopItem(db, eq.AuraID); err == nil {
			result["aura"] = item
		}
	}
	if eq.BackgroundID != "" {
		if item, err := GetShopItem(db, eq.BackgroundID); err == nil {
			result["background"] = item
		}
	}

	return result, nil
}
