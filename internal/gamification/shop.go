package gamification

import (
	"database/sql"
	"fmt"
)

// Rarity levels
const (
	RarityCommon    = "common"
	RarityUncommon  = "uncommon"
	RarityRare      = "rare"
	RarityLegendary = "legendary"
)

// Slot types
const (
	SlotHat        = "hat"
	SlotHeld       = "held"
	SlotAura       = "aura"
	SlotBackground = "background"
)

// ShopItem represents an item in the shop
type ShopItem struct {
	ID          string
	Name        string
	Description string
	Slot        string
	Price       int
	Rarity      string
	UnlockLevel int
	SpriteData  string
	Owned       bool // Computed field
}

// GetShopItems returns all items, optionally filtered by slot
func GetShopItems(db *sql.DB, slot string, userLevel int) ([]ShopItem, error) {
	query := `
		SELECT s.id, s.name, s.description, s.slot, s.price, s.rarity, s.unlock_level,
		       COALESCE(s.sprite_data, ''),
		       CASE WHEN i.item_id IS NOT NULL THEN 1 ELSE 0 END as owned
		FROM shop_items s
		LEFT JOIN inventory i ON s.id = i.item_id
	`
	args := []interface{}{}

	if slot != "" {
		query += " WHERE s.slot = ?"
		args = append(args, slot)
	}

	query += " ORDER BY s.rarity, s.price"

	rows, err := db.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []ShopItem
	for rows.Next() {
		var item ShopItem
		var owned int
		err := rows.Scan(&item.ID, &item.Name, &item.Description, &item.Slot,
			&item.Price, &item.Rarity, &item.UnlockLevel, &item.SpriteData, &owned)
		if err != nil {
			return nil, err
		}
		item.Owned = owned == 1
		items = append(items, item)
	}
	return items, rows.Err()
}

// GetShopItem returns a single item by ID
func GetShopItem(db *sql.DB, itemID string) (*ShopItem, error) {
	item := &ShopItem{}
	var owned int
	err := db.QueryRow(`
		SELECT s.id, s.name, s.description, s.slot, s.price, s.rarity, s.unlock_level,
		       COALESCE(s.sprite_data, ''),
		       CASE WHEN i.item_id IS NOT NULL THEN 1 ELSE 0 END as owned
		FROM shop_items s
		LEFT JOIN inventory i ON s.id = i.item_id
		WHERE s.id = ?
	`, itemID).Scan(&item.ID, &item.Name, &item.Description, &item.Slot,
		&item.Price, &item.Rarity, &item.UnlockLevel, &item.SpriteData, &owned)
	if err != nil {
		return nil, err
	}
	item.Owned = owned == 1
	return item, nil
}

// CanPurchase checks if user can buy an item
func CanPurchase(db *sql.DB, itemID string, userLevel int) (bool, string) {
	item, err := GetShopItem(db, itemID)
	if err != nil {
		return false, "item not found"
	}

	if item.Owned {
		return false, "already owned"
	}

	if item.UnlockLevel > userLevel {
		return false, fmt.Sprintf("requires level %d", item.UnlockLevel)
	}

	balance, err := GetBalance(db)
	if err != nil {
		return false, "wallet error"
	}

	if balance < item.Price {
		return false, fmt.Sprintf("need %d coins, have %d", item.Price, balance)
	}

	return true, ""
}

// PurchaseItem buys an item and adds to inventory
func PurchaseItem(db *sql.DB, itemID string, userLevel int) error {
	canBuy, reason := CanPurchase(db, itemID, userLevel)
	if !canBuy {
		return fmt.Errorf("cannot purchase: %s", reason)
	}

	item, err := GetShopItem(db, itemID)
	if err != nil {
		return err
	}

	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Deduct coins
	var balance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&balance)
	if err != nil {
		return err
	}

	newBalance := balance - item.Price
	_, err = tx.Exec(`UPDATE wallet SET coins = ? WHERE id = 'default'`, newBalance)
	if err != nil {
		return err
	}

	// Log transaction
	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, -item.Price, ReasonPurchase, itemID, newBalance)
	if err != nil {
		return err
	}

	// Add to inventory
	_, err = tx.Exec(`INSERT INTO inventory (item_id) VALUES (?)`, itemID)
	if err != nil {
		return err
	}

	// Log purchase
	_, err = tx.Exec(`
		INSERT INTO purchase_history (item_id, price_paid)
		VALUES (?, ?)
	`, itemID, item.Price)
	if err != nil {
		return err
	}

	return tx.Commit()
}

// GetInventory returns all owned item IDs
func GetInventory(db *sql.DB) ([]string, error) {
	rows, err := db.Query(`SELECT item_id FROM inventory ORDER BY acquired_at`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []string
	for rows.Next() {
		var itemID string
		if err := rows.Scan(&itemID); err != nil {
			return nil, err
		}
		items = append(items, itemID)
	}
	return items, rows.Err()
}

// GetOwnedItems returns full item details for inventory
func GetOwnedItems(db *sql.DB) ([]ShopItem, error) {
	rows, err := db.Query(`
		SELECT s.id, s.name, s.description, s.slot, s.price, s.rarity, s.unlock_level,
		       COALESCE(s.sprite_data, '')
		FROM shop_items s
		INNER JOIN inventory i ON s.id = i.item_id
		ORDER BY s.slot, s.rarity
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []ShopItem
	for rows.Next() {
		var item ShopItem
		err := rows.Scan(&item.ID, &item.Name, &item.Description, &item.Slot,
			&item.Price, &item.Rarity, &item.UnlockLevel, &item.SpriteData)
		if err != nil {
			return nil, err
		}
		item.Owned = true
		items = append(items, item)
	}
	return items, rows.Err()
}

// OwnsItem checks if user owns a specific item
func OwnsItem(db *sql.DB, itemID string) bool {
	var count int
	db.QueryRow(`SELECT COUNT(*) FROM inventory WHERE item_id = ?`, itemID).Scan(&count)
	return count > 0
}

// GetInventoryCount returns number of owned items
func GetInventoryCount(db *sql.DB) (int, error) {
	var count int
	err := db.QueryRow(`SELECT COUNT(*) FROM inventory`).Scan(&count)
	return count, err
}

// OwnsLegendary checks if user owns any legendary item
func OwnsLegendary(db *sql.DB) bool {
	var count int
	db.QueryRow(`
		SELECT COUNT(*) FROM inventory i
		JOIN shop_items s ON i.item_id = s.id
		WHERE s.rarity = 'legendary'
	`).Scan(&count)
	return count > 0
}
